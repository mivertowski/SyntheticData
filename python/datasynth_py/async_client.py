"""Async Python client for DataSynth."""

from __future__ import annotations

import asyncio
import json
import os
import tempfile
from dataclasses import dataclass, field
from pathlib import Path
from typing import Any, AsyncIterator, Optional

from datasynth_py.client import DataSynth, GenerationResult, OutputSpec


@dataclass
class StreamEvent:
    """A single event from a streaming generation."""

    event_type: str
    data: dict[str, Any]
    timestamp: Optional[str] = None


class AsyncDataSynth:
    """Async wrapper around the DataSynth CLI.

    Provides asyncio-compatible generation and streaming.

    Usage::

        async with AsyncDataSynth() as ds:
            result = await ds.generate(config=my_config, output={"format": "csv", "sink": "temp_dir"})

        # Or without context manager:
        ds = AsyncDataSynth()
        result = await ds.generate(config=my_config, output={"format": "csv", "sink": "temp_dir"})
    """

    def __init__(
        self,
        binary_path: Optional[str] = None,
        timeout: float = 600.0,
    ) -> None:
        """Initialize the async client.

        Args:
            binary_path: Path to the datasynth-data binary. Auto-detected if None.
            timeout: Maximum time in seconds for generation (default: 600s).
        """
        self._sync_client = DataSynth(binary_path=binary_path)
        self._timeout = timeout
        self._process: Optional[asyncio.subprocess.Process] = None

    async def __aenter__(self) -> "AsyncDataSynth":
        """Enter async context manager."""
        return self

    async def __aexit__(
        self,
        exc_type: Optional[type],
        exc_val: Optional[BaseException],
        exc_tb: Any,
    ) -> None:
        """Exit async context manager — terminate any running subprocess."""
        await self.close()

    async def close(self) -> None:
        """Terminate any running subprocess."""
        if self._process is not None:
            try:
                self._process.terminate()
                await asyncio.wait_for(self._process.wait(), timeout=5.0)
            except (asyncio.TimeoutError, ProcessLookupError):
                self._process.kill()
            finally:
                self._process = None

    async def generate(
        self,
        config: Any = None,
        output: Optional[dict[str, str]] = None,
        *,
        demo: bool = False,
        seed: Optional[int] = None,
        banking: bool = False,
        audit: bool = False,
    ) -> GenerationResult:
        """Asynchronously generate synthetic data.

        Args:
            config: Configuration object (Config instance or dict).
            output: Output specification dict with "format" and "sink" keys.
            demo: Use demo preset for quick testing.
            seed: Random seed for reproducibility.
            banking: Enable banking KYC/AML data.
            audit: Enable audit data generation.

        Returns:
            GenerationResult with output paths and metadata.

        Raises:
            RuntimeError: If generation fails.
            asyncio.TimeoutError: If generation exceeds timeout.
        """
        # Build command args
        args = self._build_generate_args(config, output, demo=demo, seed=seed, banking=banking, audit=audit)

        try:
            process = await asyncio.create_subprocess_exec(
                *args,
                stdout=asyncio.subprocess.PIPE,
                stderr=asyncio.subprocess.PIPE,
            )
            self._process = process

            stdout, stderr = await asyncio.wait_for(
                process.communicate(),
                timeout=self._timeout,
            )

            if process.returncode != 0:
                error_msg = stderr.decode("utf-8", errors="replace").strip()
                raise RuntimeError(
                    f"Generation failed (exit code {process.returncode}): {error_msg}"
                )

            # Parse output to find result directory
            output_text = stdout.decode("utf-8", errors="replace")
            return self._parse_result(output_text, output)

        finally:
            self._process = None

    async def stream_generate(
        self,
        config: Any = None,
        output: Optional[dict[str, str]] = None,
        *,
        ws_url: str = "ws://localhost:3000/ws/events",
    ) -> AsyncIterator[StreamEvent]:
        """Stream generation events via WebSocket.

        Requires the DataSynth server to be running and the ``websockets``
        package to be installed.

        Args:
            config: Configuration object.
            output: Output specification.
            ws_url: WebSocket URL for event streaming.

        Yields:
            StreamEvent objects with generation progress and data.
        """
        try:
            import websockets
        except ImportError:
            raise ImportError(
                "The 'websockets' package is required for streaming. "
                "Install with: pip install 'datasynth-py[streaming]'"
            )

        async with websockets.connect(ws_url) as ws:
            async for message in ws:
                try:
                    data = json.loads(message)
                    yield StreamEvent(
                        event_type=data.get("type", "unknown"),
                        data=data.get("data", {}),
                        timestamp=data.get("timestamp"),
                    )
                except json.JSONDecodeError:
                    yield StreamEvent(
                        event_type="raw",
                        data={"message": str(message)},
                    )

    async def validate_config(self, config: Any) -> dict[str, Any]:
        """Validate a configuration asynchronously.

        Args:
            config: Configuration object to validate.

        Returns:
            Dict with "valid" bool and optional "errors" list.
        """
        # Write config to temp file
        config_dict = config.to_dict() if hasattr(config, "to_dict") else config
        with tempfile.NamedTemporaryFile(
            mode="w", suffix=".yaml", delete=False
        ) as f:
            import yaml  # type: ignore[import-untyped]

            yaml.dump(config_dict, f)
            config_path = f.name

        try:
            binary = self._sync_client._find_binary()
            process = await asyncio.create_subprocess_exec(
                binary,
                "validate",
                "--config",
                config_path,
                stdout=asyncio.subprocess.PIPE,
                stderr=asyncio.subprocess.PIPE,
            )
            stdout, stderr = await asyncio.wait_for(
                process.communicate(), timeout=30.0
            )

            if process.returncode == 0:
                return {"valid": True, "errors": []}
            else:
                error_text = stderr.decode("utf-8", errors="replace").strip()
                return {"valid": False, "errors": [error_text]}
        finally:
            os.unlink(config_path)

    def _build_generate_args(
        self,
        config: Any,
        output: Optional[dict[str, str]],
        *,
        demo: bool = False,
        seed: Optional[int] = None,
        banking: bool = False,
        audit: bool = False,
    ) -> list[str]:
        """Build CLI arguments for generation."""
        binary = self._sync_client._find_binary()
        args = [binary, "generate"]

        if demo:
            args.append("--demo")

        if seed is not None:
            args.extend(["--seed", str(seed)])

        if banking:
            args.append("--banking")

        if audit:
            args.append("--audit")

        # Handle output directory
        if output and output.get("sink") == "temp_dir":
            output_dir = tempfile.mkdtemp(prefix="datasynth_")
            args.extend(["--output", output_dir])
        elif output and "directory" in output:
            args.extend(["--output", output["directory"]])
        else:
            output_dir = tempfile.mkdtemp(prefix="datasynth_")
            args.extend(["--output", output_dir])

        # Handle config
        if config is not None and not demo:
            config_dict = config.to_dict() if hasattr(config, "to_dict") else config
            if isinstance(config_dict, dict):
                with tempfile.NamedTemporaryFile(
                    mode="w", suffix=".yaml", delete=False
                ) as f:
                    import yaml  # type: ignore[import-untyped]

                    yaml.dump(config_dict, f)
                    args.extend(["--config", f.name])

        return args

    def _parse_result(
        self, output_text: str, output_spec: Optional[dict[str, str]]
    ) -> GenerationResult:
        """Parse generation output into a result object."""
        # Find the output directory from args or output
        output_dir = None
        if output_spec:
            output_dir = output_spec.get("directory")

        return GenerationResult(
            output_dir=output_dir or "",
            stdout=output_text,
            success=True,
        )
