import { useEffect, useState } from "react";

type MooncakeSource = 
  | { MooncakesIO: { name: string; version?: string } }
  | { Git: { url: string; rev?: string } };

type MoonCommand = "Check" | "Build" | "Test";

type ToolChainLabel = "Stable" | "Bleeding";

interface ToolChainVersion {
  label: ToolChainLabel;
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
        <h1 className="text-2xl font-bold mb-4">Moon Build Dashboard</h1>

        {error ? (
          <p className="text-red-500 text-center">{error}</p>
        ) : data ? (
          <div className="overflow-x-auto">
            <table className="min-w-full table-auto bg-white shadow-md rounded-lg overflow-hidden">
              <thead>
                <tr className="bg-gray-200">
                  <th rowSpan={2} className="py-2 px-4 text-left w-1/4 border-r">Repository</th>
                  <th colSpan={4} className="py-2 px-4 text-center w-1/2 bg-blue-500 text-white border-r">
                    Stable
                    <div className="text-xs mt-1 font-normal">
                      {data.stable_toolchain_version.moon_version} / {data.stable_toolchain_version.moonc_version}
                    </div>
                  </th>
                  <th colSpan={4} className="py-2 px-4 text-center w-1/2 bg-green-500 text-white">
                    Bleeding
                    <div className="text-xs mt-1 font-normal">
                      {data.bleeding_toolchain_version.moon_version} / {data.bleeding_toolchain_version.moonc_version}
                    </div>
                  </th>
                </tr>
                <tr className="bg-gray-100">
                  <th className="py-1 px-4 text-left text-sm border-r">Check</th>
                  <th className="py-1 px-4 text-left text-sm border-r">Build</th>
                  <th className="py-1 px-4 text-left text-xs border-r">Test (build only)</th>
                  <th className="py-1 px-4 text-left text-xs border-r">Start Time</th>
                  <th className="py-1 px-4 text-left text-sm border-r">Check</th>
                  <th className="py-1 px-4 text-left text-sm border-r">Build</th>
                  <th className="py-1 px-4 text-left text-xs border-r">Test (build only)</th>
                  <th className="py-1 px-4 text-left text-xs">Start Time</th>
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
                    <td className={`py-2 px-4 ${getStatusStyle(entry.check.status)} border-r`}>
                      {getStatusText(entry.check.status, entry.check.elapsed)}
                    </td>
                    <td className={`py-2 px-4 ${getStatusStyle(entry.build.status)} border-r`}>
                      {getStatusText(entry.build.status, entry.build.elapsed)}
                    </td>
                    <td className={`py-2 px-4 ${getStatusStyle(entry.test.status)} border-r text-xs`}>
                      {getStatusText(entry.test.status, entry.test.elapsed)}
                    </td>
                    <td className="py-2 px-4 text-xs border-r">{entry.test.start_time}</td>
                    <td className={`py-2 px-4 ${getStatusStyle(data.bleeding_release_data[index].check.status)} border-r`}>
                      {getStatusText(data.bleeding_release_data[index].check.status, data.bleeding_release_data[index].check.elapsed)}
                    </td>
                    <td className={`py-2 px-4 ${getStatusStyle(data.bleeding_release_data[index].build.status)} border-r`}>
                      {getStatusText(data.bleeding_release_data[index].build.status, data.bleeding_release_data[index].build.elapsed)}
                    </td>
                    <td className={`py-2 px-4 ${getStatusStyle(data.bleeding_release_data[index].test.status)} border-r text-xs`}>
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
