import type { ReactNode } from "react";

export interface DataTableColumn<T> {
  key: string;
  title: string;
  render: (item: T) => ReactNode;
}

export function DataTable<T>({ columns, data }: { columns: DataTableColumn<T>[]; data: T[] }) {
  return (
    <div className="table-shell">
      <table>
        <thead>
          <tr>
            {columns.map((column) => (
              <th key={column.key}>{column.title}</th>
            ))}
          </tr>
        </thead>
        <tbody>
          {data.map((item, index) => (
            <tr key={index}>
              {columns.map((column) => (
                <td key={column.key}>{column.render(item)}</td>
              ))}
            </tr>
          ))}
        </tbody>
      </table>
    </div>
  );
}
