import { useEffect, useState } from "react";

type MooncakeSource = 
  | { MooncakesIO: { name: string; version?: string } }
  | { Git: { url: string; rev?: string } };

type MoonCommand = "Check" | "Build" | "Test";

interface ToolChainVersion {
  label: string;
  moon_version: string;
  moonc_version: string;
}

interface MoonBuildDashboard {
  run_id: string;
  run_number: string;
  start_time: string;
  sources: MooncakeSource[];
  stable_toolchain_version: ToolChainVersion;
  stable_release_data: BuildState[];
  bleeding_toolchain_version: ToolChainVersion;
  bleeding_release_data: BuildState[];
}

type Status = "Success" | "Failure";

interface ExecuteResult {
  status: Status;
  start_time: string;
  elapsed: number;
}

interface BuildState {
  source: MooncakeSource;
  check: ExecuteResult;
  build: ExecuteResult;
  test: ExecuteResult;
}

async function get_data(): Promise<MoonBuildDashboard> {
  const response = await fetch('/data.jsonl');
  const text = await response.text();
  const lines = text.split('\n').filter((line) => line.trim() !== '');
  const parsedData = lines.map((line) => JSON.parse(line));
  return parsedData[parsedData.length - 1];
}

const App = () => {
  const [data, setData] = useState<MoonBuildDashboard | null>(null);
  const [error, setError] = useState<string | null>(null);

  const fetchData = async () => {
    try {
      const parsedData = await get_data();
      setData(parsedData);
    } catch (err) {
      if (err instanceof Error) {
        setError(err.message);
      } else {
        setError("Unknown error occurred");
      }
    }
  };

  useEffect(() => {
    fetchData();
  }, []);

  const getStatusStyle = (status: Status): string => {
    return status === "Success"
      ? "bg-green-200 text-green-800"
      : "bg-red-200 text-red-800";
  };

  const getStatusText = (status: Status, elapsed: number | null): string => {
    return status === "Success" ? `${elapsed ?? '-'} ms` : "Failed";
  };

  return (
    <div className="p-4 bg-gray-100 min-h-screen flex justify-center">
      <div className="max-w-[1200px] w-full">
        <h1 className="text-2xl font-bold mb-2">Moon Build Dashboard</h1>

        <div className="mb-4">
          <p className="font-mono">moon version: {data?.stable_toolchain_version.moon_version}</p>
          <p className="font-mono">moonc version: {data?.stable_toolchain_version.moonc_version}</p>
          <p className="font-mono text-xs">
            GitHub Action:{" "}
            <a
              href={`https://github.com/moonbitlang/moon-build-dashboard/actions/runs/${data?.run_id}`}
              target="_blank"
              rel="noopener noreferrer"
            >
              {`https://github.com/moonbitlang/moon-build-dashboard/actions/runs/${data?.run_id}`}
            </a>
          </p>
        </div>

        {error ? (
          <p className="text-red-500 text-center">{error}</p>
        ) : data ? (
          <div className="overflow-x-auto">
            <table className="min-w-full table-auto bg-white shadow-md rounded-lg overflow-hidden">
              <thead className="bg-blue-500 text-white">
                <tr>
                  <th className="py-2 px-4 text-left w-1/4">Repository</th>
                  <th className="py-2 px-4 text-left w-1/12">Stable Check</th>
                  <th className="py-2 px-4 text-left w-1/12">Stable Build</th>
                  <th className="py-2 px-4 text-left w-1/12">
                    <div>
                      Stable Test<div className="text-xs">(build only)</div>
                    </div>
                  </th>
                  <th className="py-2 px-4 text-left w-1/12">Stable Start Time</th>
                  <th className="py-2 px-4 text-left w-1/12">Bleeding Check</th>
                  <th className="py-2 px-4 text-left w-1/12">Bleeding Build</th>
                  <th className="py-2 px-4 text-left w-1/12">
                    <div>
                      Bleeding Test<div className="text-xs">(build only)</div>
                    </div>
                  </th>
                  <th className="py-2 px-4 text-left w-1/12">Bleeding Start Time</th>
                </tr>
              </thead>
              <tbody>
                {data.stable_release_data.map((entry, index) => (
                  <tr key={index} className="border-b hover:bg-gray-50">
                    <td className="py-2 px-4">
                      <a
                        href={
                          "Git" in entry.source
                            ? entry.source.Git.url
                            : "#"
                        }
                        className="text-blue-600 hover:text-blue-800"
                        target="_blank"
                        rel="noopener noreferrer"
                      >
                        {entry.source && "Git" in entry.source
                          ? entry.source.Git.url.replace("https://github.com/", "")
                          : ""}
                      </a>
                    </td>
                    <td className={`py-2 px-4 ${getStatusStyle(entry.check.status)}`}>
                      {getStatusText(entry.check.status, entry.check.elapsed)}
                    </td>
                    <td className={`py-2 px-4 ${getStatusStyle(entry.build.status)}`}>
                      {getStatusText(entry.build.status, entry.build.elapsed)}
                    </td>
                    <td className={`py-2 px-4 ${getStatusStyle(entry.test.status)}`}>
                      {getStatusText(entry.test.status, entry.test.elapsed)}
                    </td>
                    <td className="py-2 px-4 text-xs">{entry.test.start_time}</td>
                    <td className={`py-2 px-4 ${getStatusStyle(data.bleeding_release_data[index].check.status)}`}>
                      {getStatusText(data.bleeding_release_data[index].check.status, data.bleeding_release_data[index].check.elapsed)}
                    </td>
                    <td className={`py-2 px-4 ${getStatusStyle(data.bleeding_release_data[index].build.status)}`}>
                      {getStatusText(data.bleeding_release_data[index].build.status, data.bleeding_release_data[index].build.elapsed)}
                    </td>
                    <td className={`py-2 px-4 ${getStatusStyle(data.bleeding_release_data[index].test.status)}`}>
                      {getStatusText(data.bleeding_release_data[index].test.status, data.bleeding_release_data[index].test.elapsed)}
                    </td>
                    <td className="py-2 px-4 text-xs">{data.bleeding_release_data[index].test.start_time}</td>
                  </tr>
                ))}
              </tbody>
            </table>
          </div>
        ) : (
          <p>Loading...</p>
        )}
      </div>
    </div>
  );
};

export default App;
