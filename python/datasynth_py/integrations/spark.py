"""Apache Spark connector for DataSynth synthetic data.

Provides:
- DataSynthSparkReader: Read DataSynth output as Spark DataFrames
"""

from __future__ import annotations

from pathlib import Path
from typing import Any, Dict, List, Optional

try:
    from pyspark.sql import DataFrame, SparkSession
    from pyspark.sql.types import (
        BooleanType,
        DateType,
        DecimalType,
        DoubleType,
        IntegerType,
        LongType,
        StringType,
        StructField,
        StructType,
        TimestampType,
    )

    HAS_PYSPARK = True
except ImportError:
    HAS_PYSPARK = False
    DataFrame = None  # type: ignore[assignment, misc]
    SparkSession = None  # type: ignore[assignment, misc]


# Column name pattern to Spark type mapping
_SPARK_TYPE_MAP: Dict[str, str] = {
    "id": "string",
    "uuid": "string",
    "code": "string",
    "name": "string",
    "text": "string",
    "description": "string",
    "reference": "string",
    "type": "string",
    "status": "string",
    "currency": "string",
    "country": "string",
    "date": "date",
    "posting_date": "date",
    "document_date": "date",
    "created_at": "timestamp",
    "timestamp": "timestamp",
    "amount": "decimal",
    "debit_amount": "decimal",
    "credit_amount": "decimal",
    "balance": "decimal",
    "total": "decimal",
    "price": "decimal",
    "value": "decimal",
    "rate": "double",
    "weight": "double",
    "score": "double",
    "probability": "double",
    "count": "integer",
    "year": "integer",
    "period": "integer",
    "month": "integer",
    "day": "integer",
    "level": "integer",
    "quantity": "integer",
    "is_": "boolean",
    "enabled": "boolean",
    "active": "boolean",
    "has_": "boolean",
}


def _infer_spark_type(column_name: str) -> str:
    """Infer Spark SQL type from column name patterns.

    Args:
        column_name: The CSV column header name.

    Returns:
        Spark SQL type string (e.g., "string", "decimal", "date").
    """
    col_lower = column_name.lower()
    for pattern, spark_type in _SPARK_TYPE_MAP.items():
        if pattern in col_lower:
            return spark_type
    return "string"


def _read_csv_headers(csv_path: Path) -> List[str]:
    """Read column headers from a CSV file."""
    with open(csv_path, "r") as f:
        first_line = f.readline().strip()
    return [col.strip().strip('"') for col in first_line.split(",")]


class DataSynthSparkReader:
    """Read DataSynth output files as Apache Spark DataFrames.

    Provides methods to read individual tables, all tables at once,
    or register them as temporary SQL views for querying.

    Raises:
        ImportError: If pyspark is not installed.

    Example::

        reader = DataSynthSparkReader()
        spark = SparkSession.builder.appName("datasynth").getOrCreate()

        # Read a single table
        df = reader.read_table(spark, "./output", "journal_entries")

        # Read all tables
        tables = reader.read_all_tables(spark, "./output")

        # Register as SQL views
        views = reader.create_temp_views(spark, "./output")
        spark.sql("SELECT * FROM journal_entries LIMIT 10").show()
    """

    def __init__(self):
        if not HAS_PYSPARK:
            raise ImportError(
                "PySpark is required for DataSynth Spark integration. "
                "Install with: pip install 'datasynth-py[spark]'"
            )

    def read_table(
        self,
        spark: Any,
        output_path: str,
        table_name: str,
        infer_schema: bool = True,
        custom_schema: Optional[Any] = None,
    ) -> Any:
        """Read a single DataSynth CSV table as a Spark DataFrame.

        Args:
            spark: Active SparkSession instance.
            output_path: Path to DataSynth output directory.
            table_name: Name of the table (CSV filename without extension).
            infer_schema: Whether to infer types from column names. Default: True.
            custom_schema: Optional StructType to override schema inference.

        Returns:
            Spark DataFrame containing the table data.

        Raises:
            FileNotFoundError: If the CSV file does not exist.
        """
        output_dir = Path(output_path)
        csv_path = output_dir / f"{table_name}.csv"

        # Also check subdirectories
        if not csv_path.exists():
            matches = list(output_dir.rglob(f"{table_name}.csv"))
            if not matches:
                raise FileNotFoundError(
                    f"Table '{table_name}' not found at {csv_path} or in subdirectories "
                    f"of {output_dir}"
                )
            csv_path = matches[0]

        reader = spark.read.option("header", "true")

        if custom_schema is not None:
            reader = reader.schema(custom_schema)
        elif infer_schema:
            schema = self._build_schema(csv_path)
            if schema is not None:
                reader = reader.schema(schema)
            else:
                reader = reader.option("inferSchema", "true")
        else:
            reader = reader.option("inferSchema", "true")

        return reader.csv(str(csv_path))

    def read_all_tables(
        self,
        spark: Any,
        output_path: str,
        infer_schema: bool = True,
    ) -> Dict[str, Any]:
        """Read all DataSynth CSV tables as Spark DataFrames.

        Args:
            spark: Active SparkSession instance.
            output_path: Path to DataSynth output directory.
            infer_schema: Whether to infer types from column names.

        Returns:
            Dict mapping table names to Spark DataFrames.
        """
        output_dir = Path(output_path)
        if not output_dir.exists():
            raise FileNotFoundError(f"Output directory not found: {output_dir}")

        csv_files = sorted(output_dir.rglob("*.csv"))
        tables: Dict[str, Any] = {}

        for csv_file in csv_files:
            table_name = csv_file.stem
            try:
                df = self.read_table(
                    spark, output_path, table_name, infer_schema=infer_schema
                )
                tables[table_name] = df
            except Exception:
                # Skip files that cannot be read
                continue

        return tables

    def create_temp_views(
        self,
        spark: Any,
        output_path: str,
        infer_schema: bool = True,
    ) -> List[str]:
        """Read all tables and register them as Spark temporary SQL views.

        After calling this method, tables can be queried using Spark SQL:
        ``spark.sql("SELECT * FROM journal_entries")``

        Args:
            spark: Active SparkSession instance.
            output_path: Path to DataSynth output directory.
            infer_schema: Whether to infer types from column names.

        Returns:
            List of registered view names.
        """
        tables = self.read_all_tables(spark, output_path, infer_schema=infer_schema)
        view_names: List[str] = []

        for table_name, df in tables.items():
            # Sanitize table name for SQL (replace hyphens, spaces)
            safe_name = table_name.replace("-", "_").replace(" ", "_")
            df.createOrReplaceTempView(safe_name)
            view_names.append(safe_name)

        return sorted(view_names)

    @staticmethod
    def _build_schema(csv_path: Path) -> Optional[Any]:
        """Build a Spark StructType schema from CSV headers using type inference.

        Args:
            csv_path: Path to the CSV file.

        Returns:
            StructType schema, or None if headers cannot be read.
        """
        if not HAS_PYSPARK:
            return None

        try:
            headers = _read_csv_headers(csv_path)
        except (OSError, UnicodeDecodeError):
            return None

        if not headers:
            return None

        type_mapping = {
            "string": StringType(),
            "date": DateType(),
            "timestamp": TimestampType(),
            "decimal": DecimalType(18, 2),
            "double": DoubleType(),
            "integer": IntegerType(),
            "long": LongType(),
            "boolean": BooleanType(),
        }

        fields = []
        for header in headers:
            inferred = _infer_spark_type(header)
            spark_type = type_mapping.get(inferred, StringType())
            fields.append(StructField(header, spark_type, nullable=True))

        return StructType(fields)
