interface StatusSelectProps {
  status: string | null;
  onChange: (status: string) => void;
}

const STATUS_OPTIONS: { value: string; label: string; color: string }[] = [
  { value: "", label: "No status", color: "" },
  {
    value: "want_to_read",
    label: "Want to Read",
    color: "bg-blue-100 text-blue-800 dark:bg-blue-900/40 dark:text-blue-300",
  },
  {
    value: "reading",
    label: "Reading",
    color:
      "bg-green-100 text-green-800 dark:bg-green-900/40 dark:text-green-300",
  },
  {
    value: "finished",
    label: "Finished",
    color:
      "bg-amber-100 text-amber-800 dark:bg-amber-900/40 dark:text-amber-300",
  },
  {
    value: "abandoned",
    label: "Abandoned",
    color: "bg-red-100 text-red-800 dark:bg-red-900/40 dark:text-red-300",
  },
];

export function getStatusInfo(status: string | null) {
  return STATUS_OPTIONS.find((o) => o.value === status) ?? STATUS_OPTIONS[0];
}

export default function StatusSelect({
  status,
  onChange,
}: StatusSelectProps) {
  return (
    <select
      value={status ?? ""}
      onChange={(e) => onChange(e.target.value)}
      className="rounded-md border border-gray-300 bg-white px-3 py-1.5 text-sm
        text-gray-900 dark:border-gray-600 dark:bg-gray-800 dark:text-gray-100
        focus:ring-2 focus:ring-amber-500 focus:outline-none"
    >
      {STATUS_OPTIONS.map((opt) => (
        <option key={opt.value} value={opt.value}>
          {opt.label}
        </option>
      ))}
    </select>
  );
}
