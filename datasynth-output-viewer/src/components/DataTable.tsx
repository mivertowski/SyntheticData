import { useId, useMemo, useState, useEffect } from 'react';
import './DataTable.css';

const DEFAULT_PAGE_SIZE_OPTIONS = [50, 100, 200, 500];

interface DataTableProps<T extends Record<string, unknown>> {
  data: T[];
  columns: { key: keyof T | string; label: string; width?: string; format?: (v: unknown) => string }[];
  keyField?: keyof T | string;
  pageSize?: number;
  /** Options for rows-per-page selector; default [50, 100, 200, 500]. Pass [] to hide. */
  pageSizeOptions?: number[];
  maxHeight?: string;
  /** Optional row click handler (e.g. to select row for detail view) */
  onRowClick?: (row: T) => void;
  /** Optional class name for the selected row (when onRowClick is used and row is selected) */
  selectedRowKey?: string | null;
}

export function DataTable<T extends Record<string, unknown>>({
  data,
  columns,
  keyField,
  pageSize = 50,
  pageSizeOptions = DEFAULT_PAGE_SIZE_OPTIONS,
  maxHeight = '60vh',
  onRowClick,
  selectedRowKey = null,
}: DataTableProps<T>) {
  const paginationId = useId();
  const [page, setPage] = useState(0);
  const [currentPageSize, setCurrentPageSize] = useState(() =>
    pageSizeOptions.length && pageSizeOptions.includes(pageSize) ? pageSize : (pageSizeOptions[0] ?? pageSize)
  );
  const totalPages = Math.max(1, Math.ceil(data.length / currentPageSize));
  useEffect(() => {
    if (pageSizeOptions.length && !pageSizeOptions.includes(currentPageSize)) {
      setCurrentPageSize(pageSizeOptions[0] ?? 50);
    }
  }, [pageSizeOptions, currentPageSize]);
  useEffect(() => {
    if (page >= totalPages && totalPages > 0) setPage(totalPages - 1);
  }, [page, totalPages]);
  useEffect(() => {
    setPage(0);
  }, [data]);
  const slice = useMemo(
    () => data.slice(page * currentPageSize, (page + 1) * currentPageSize),
    [data, page, currentPageSize]
  );

  const getVal = (row: T, key: keyof T | string): unknown => {
    const k = key as string;
    return k in row ? row[k] : '';
  };

  return (
    <div className="data-table-wrap">
      <div className="data-table-scroll" style={{ maxHeight }}>
        <table className="data-table">
          <thead>
            <tr>
              {columns.map((col) => (
                <th key={String(col.key)} style={col.width ? { width: col.width } : undefined}>
                  {col.label}
                </th>
              ))}
            </tr>
          </thead>
          <tbody>
            {slice.map((row, i) => {
              const key = keyField ? String(getVal(row, keyField)) || `row-${page * currentPageSize + i}` : `row-${page * currentPageSize + i}`;
              const isSelected = selectedRowKey != null && key === selectedRowKey;
              return (
              <tr
                key={key}
                className={onRowClick ? (isSelected ? 'data-table-row-selectable data-table-row-selected' : 'data-table-row-selectable') : ''}
                onClick={onRowClick ? () => onRowClick(row) : undefined}
                role={onRowClick ? 'button' : undefined}
              >
                {columns.map((col) => {
                  const v = getVal(row, col.key);
                  return (
                    <td key={String(col.key)}>
                      {col.format ? col.format(v) : v != null ? String(v) : ''}
                    </td>
                  );
                })}
              </tr>
            );
            })}
          </tbody>
        </table>
      </div>
      {(totalPages > 1 || pageSizeOptions.length > 0) && (
        <div className="data-table-pagination">
          {pageSizeOptions.length > 0 && (
            <div className="data-table-page-size">
              <label htmlFor={paginationId}>Rows per page:</label>
              <select
                id={paginationId}
                value={currentPageSize}
                onChange={(e) => {
                  const v = Number(e.target.value);
                  if (Number.isFinite(v) && v > 0) {
                    setCurrentPageSize(v);
                    setPage(0);
                  }
                }}
              >
                {pageSizeOptions.map((n) => (
                  <option key={n} value={n}>
                    {n}
                  </option>
                ))}
              </select>
            </div>
          )}
          {totalPages > 1 && (
            <>
              <button
                type="button"
                disabled={page === 0}
                onClick={() => setPage((p) => Math.max(0, p - 1))}
              >
                Previous
              </button>
              <span>
                Page {page + 1} of {totalPages} ({data.length} rows)
              </span>
              <button
                type="button"
                disabled={page >= totalPages - 1}
                onClick={() => setPage((p) => Math.min(totalPages - 1, p + 1))}
              >
                Next
              </button>
            </>
          )}
        </div>
      )}
    </div>
  );
}
