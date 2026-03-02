"""Client entrypoint for the DataSynth Python wrapper."""

from __future__ import annotations

import json
import os
import pathlib
import subprocess
import tempfile
import urllib.error
import urllib.request
from dataclasses import dataclass, field
from typing import Any, AsyncIterator, Dict, List, Optional

import importlib.util

from datasynth_py.config.models import Config, MissingDependencyError


@dataclass(frozen=True)
class OutputSpec:
    format: str = "csv"
    sink: str = "temp_dir"
    path: Optional[str] = None
    compression: Optional[str] = None
    table_format: str = "pandas"


@dataclass(frozen=True)
class GenerationResult:
    output_dir: Optional[str] = None
    tables: Optional[Dict[str, Any]] = None
    metadata: Dict[str, Any] = field(default_factory=dict)


class DataSynth:
    """Python wrapper for running DataSynth generation."""

    def __init__(
        self,
        binary_path: Optional[str] = None,
        server_url: str = "http://localhost:3000",
        api_key: Optional[str] = None,
        request_timeout: float = 30.0,
    ) -> None:
        self._binary_path = binary_path or os.environ.get("DATASYNTH_BINARY", "datasynth-data")
        self._server_url = server_url.rstrip("/")
        self._api_key = api_key
        self._request_timeout = request_timeout
        self._fingerprint_client: Optional["FingerprintClient"] = None

    @property
    def fingerprint(self) -> "FingerprintClient":
        """Access fingerprint operations.

        Returns:
            FingerprintClient for extract, validate, info, evaluate operations.

        Example:
            >>> synth = DataSynth()
            >>> synth.fingerprint.extract("./data/", "./fp.dsf")
            >>> info = synth.fingerprint.info("./fp.dsf")
        """
        if self._fingerprint_client is None:
            from datasynth_py.fingerprint import FingerprintClient
            self._fingerprint_client = FingerprintClient(self._binary_path)
        return self._fingerprint_client

    def generate(
        self,
        config: Config,
        output: Optional[OutputSpec | Dict[str, Any]] = None,
        seed: Optional[int] = None,
        fraud_scenario: Optional[List[str]] = None,
        fraud_rate: Optional[float] = None,
        stream_file: Optional[str] = None,
    ) -> GenerationResult:
        config.validate()
        output_spec = _coerce_output_spec(output)
        if seed is not None:
            config = config.override(**{"global": {"seed": seed}})
        if output_spec.sink == "path" and not output_spec.path:
            raise ValueError("OutputSpec.path must be set when sink='path'.")

        output_dir = self._resolve_output_dir(output_spec)
        config_path = self._write_config(config, output_dir, output_spec)
        self._run_cli(config_path=config_path, output_dir=output_dir, fraud_scenario=fraud_scenario, fraud_rate=fraud_rate, stream_file=stream_file)

        if output_spec.sink == "memory":
            tables = _load_tables(output_dir, output_spec)
            return GenerationResult(output_dir=None, tables=tables)
        return GenerationResult(output_dir=output_dir, tables=None)

    def stream(
        self,
        config: Optional[Config] = None,
        events_per_second: Optional[int] = None,
        max_events: Optional[int] = None,
        inject_anomalies: Optional[bool] = None,
        seed: Optional[int] = None,
    ) -> "StreamingSession":
        if config is not None:
            config.validate()
            payload = _config_to_server_payload(config, seed)
            self._post_json("/api/config", payload)

        stream_payload: Dict[str, Any] = {}
        if events_per_second is not None:
            stream_payload["events_per_second"] = events_per_second
        if max_events is not None:
            stream_payload["max_events"] = max_events
        if inject_anomalies is not None:
            stream_payload["inject_anomalies"] = inject_anomalies
        self._post_json("/api/stream/start", stream_payload)
        return StreamingSession(
            server_url=self._server_url,
            api_key=self._api_key,
            request_timeout=self._request_timeout,
        )

    def _write_config(self, config: Config, output_dir: str, output_spec: OutputSpec) -> str:
        yaml_spec = importlib.util.find_spec("yaml")
        if yaml_spec is None:
            raise MissingDependencyError(
                "PyYAML is required to generate config files. Install with `pip install PyYAML`."
            )
        import yaml  # type: ignore

        payload = config.to_dict()

        # Ensure output section exists with required fields
        if "output" not in payload:
            payload["output"] = {}
        payload["output"]["output_directory"] = output_dir

        # Map output format from OutputSpec
        format_map = {"csv": "csv", "jsonl": "json", "parquet": "parquet"}
        cli_format = format_map.get(output_spec.format, "csv")
        payload["output"]["formats"] = [cli_format]

        data = yaml.safe_dump(payload, sort_keys=False)
        fd, path = tempfile.mkstemp(prefix="datasynth_", suffix=".yaml")
        os.close(fd)
        pathlib.Path(path).write_text(data, encoding="utf-8")
        return path

    def _resolve_output_dir(self, output: OutputSpec) -> str:
        if output.sink == "path" and output.path:
            return output.path
        if output.sink == "temp_dir":
            return tempfile.mkdtemp(prefix="datasynth_output_")
        if output.sink == "memory":
            return tempfile.mkdtemp(prefix="datasynth_output_")
        raise ValueError(f"Unknown output sink: {output.sink}")

    def _run_cli(self, config_path: str, output_dir: str, fraud_scenario: Optional[List[str]] = None, fraud_rate: Optional[float] = None, stream_file: Optional[str] = None) -> None:
        command = [
            self._binary_path,
            "generate",
            "--config",
            config_path,
            "--output",
            output_dir,
        ]
        if fraud_scenario:
            for pack in fraud_scenario:
                command.extend(["--fraud-scenario", pack])
        if fraud_rate is not None:
            command.extend(["--fraud-rate", str(fraud_rate)])
        if stream_file is not None:
            command.extend(["--stream-file", stream_file])
        try:
            subprocess.run(command, check=True, capture_output=True, text=True)
        except FileNotFoundError as exc:
            raise RuntimeError(
                "datasynth-data binary not found. Build it with `cargo build --release` "
                "and set DATASYNTH_BINARY or pass binary_path."
            ) from exc
        except subprocess.CalledProcessError as exc:
            raise RuntimeError(
                f"datasynth-data failed: {exc.stderr or exc.stdout}"
            ) from exc

    def _post_json(self, path: str, payload: Dict[str, Any]) -> Dict[str, Any]:
        url = f"{self._server_url}{path}"
        data = json.dumps(payload).encode("utf-8")
        headers = {"Content-Type": "application/json"}
        if self._api_key:
            headers["X-API-Key"] = self._api_key
        request = urllib.request.Request(url, data=data, headers=headers, method="POST")
        try:
            with urllib.request.urlopen(request, timeout=self._request_timeout) as response:
                body = response.read().decode("utf-8")
        except urllib.error.HTTPError as exc:
            body = exc.read().decode("utf-8")
            raise RuntimeError(f"Server error ({exc.code}): {body}") from exc
        return json.loads(body) if body else {}


@dataclass(frozen=True)
class StreamingSession:
    server_url: str
    api_key: Optional[str]
    request_timeout: float

    def pause(self) -> Dict[str, Any]:
        return self._control("/api/stream/pause")

    def resume(self) -> Dict[str, Any]:
        return self._control("/api/stream/resume")

    def stop(self) -> Dict[str, Any]:
        return self._control("/api/stream/stop")

    def trigger_pattern(self, pattern: str) -> Dict[str, Any]:
        """Trigger a pattern in the streaming session.

        Args:
            pattern: Pattern name (year_end_spike, period_end_spike, fraud_cluster, etc.)

        Returns:
            Response from the server.
        """
        return self._control(f"/api/stream/trigger/{pattern}")

    def trigger_year_end(self) -> Dict[str, Any]:
        """Trigger year-end closing patterns (high volume, accruals, adjustments)."""
        return self.trigger_pattern("year_end_spike")

    def trigger_month_end(self) -> Dict[str, Any]:
        """Trigger month-end/period-end patterns."""
        return self.trigger_pattern("period_end_spike")

    def trigger_fraud_cluster(self) -> Dict[str, Any]:
        """Trigger a cluster of fraud-related transactions."""
        return self.trigger_pattern("fraud_cluster")

    def trigger_quarter_end(self) -> Dict[str, Any]:
        """Trigger quarter-end closing patterns."""
        return self.trigger_pattern("quarter_end_spike")

    async def events(self) -> AsyncIterator[Dict[str, Any]]:
        websockets_spec = importlib.util.find_spec("websockets")
        if websockets_spec is None:
            raise MissingDependencyError(
                "The websockets package is required for streaming. Install with `pip install websockets`."
            )
        import websockets  # type: ignore

        ws_url = self.server_url.replace("http", "ws") + "/ws/events"
        headers = []
        if self.api_key:
            headers.append(("X-API-Key", self.api_key))
        async with websockets.connect(ws_url, extra_headers=headers) as websocket:
            async for message in websocket:
                yield json.loads(message)

    def _control(self, path: str) -> Dict[str, Any]:
        url = f"{self.server_url}{path}"
        headers = {"Content-Type": "application/json"}
        if self.api_key:
            headers["X-API-Key"] = self.api_key
        request = urllib.request.Request(url, data=b"{}", headers=headers, method="POST")
        with urllib.request.urlopen(request, timeout=self.request_timeout) as response:
            body = response.read().decode("utf-8")
        return json.loads(body) if body else {}


def _coerce_output_spec(value: Optional[OutputSpec | Dict[str, Any]]) -> OutputSpec:
    if value is None:
        return OutputSpec()
    if isinstance(value, OutputSpec):
        return value
    return OutputSpec(**value)


def _load_tables(output_dir: str, output_spec: OutputSpec) -> Dict[str, Any]:
    if output_spec.table_format != "pandas":
        raise ValueError("Only pandas table_format is supported in this wrapper.")
    pandas_spec = importlib.util.find_spec("pandas")
    if pandas_spec is None:
        raise MissingDependencyError(
            "pandas is required for in-memory tables. Install with `pip install pandas`."
        )
    import pandas as pd  # type: ignore

    tables: Dict[str, Any] = {}
    directory = pathlib.Path(output_dir)
    if output_spec.format == "csv":
        for csv_path in directory.rglob("*.csv"):
            tables[csv_path.stem] = pd.read_csv(csv_path)
    elif output_spec.format == "jsonl":
        for json_path in directory.rglob("*.jsonl"):
            tables[json_path.stem] = pd.read_json(json_path, lines=True)
    elif output_spec.format == "parquet":
        for parquet_path in directory.rglob("*.parquet"):
            tables[parquet_path.stem] = pd.read_parquet(parquet_path)
    else:
        raise ValueError(f"Unsupported format for memory loading: {output_spec.format}")
    return tables


def _config_to_server_payload(config: Config, seed: Optional[int]) -> Dict[str, Any]:
    """Convert Config to server API payload format."""
    payload = config.to_dict()
    global_settings = payload.get("global", {})
    companies = payload.get("companies", [])
    chart_of_accounts = payload.get("chart_of_accounts", {})
    fraud = payload.get("fraud", {})

    # Extract values from the new schema structure
    industry = global_settings.get("industry", "retail")
    complexity = chart_of_accounts.get("complexity", "small")
    start_date = global_settings.get("start_date", "2024-01-01")
    period_months = global_settings.get("period_months", 12)
    seed_value = seed if seed is not None else global_settings.get("seed")

    # Companies is now a list of company configs
    company_payloads: List[Dict[str, Any]] = []
    if isinstance(companies, list):
        for company in companies:
            company_payloads.append({
                "code": company.get("code", "C001"),
                "name": company.get("name", "Company"),
                "currency": company.get("currency", "USD"),
                "country": company.get("country", "US"),
                "annual_transaction_volume": 10000,
                "volume_weight": company.get("volume_weight", 1.0),
            })
    else:
        # Fallback for legacy format
        company_payloads.append({
            "code": "C001",
            "name": "Company 1",
            "currency": "USD",
            "country": "US",
            "annual_transaction_volume": 10000,
            "volume_weight": 1.0,
        })

    # Extract fraud settings
    fraud_enabled = fraud.get("enabled", False)
    fraud_rate = fraud.get("rate", 0.0)

    return {
        "industry": industry,
        "start_date": start_date,
        "period_months": period_months,
        "seed": seed_value,
        "coa_complexity": complexity,
        "companies": company_payloads,
        "fraud_enabled": fraud_enabled,
        "fraud_rate": fraud_rate,
    }
