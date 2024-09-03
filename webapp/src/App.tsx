import { useEffect, useState } from "react";

type MooncakeSource = 
  | { MooncakesIO: { name: string; version: string[]; index: number } }
  | { Git: { url: string; rev: string[]; index: number } };

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

interface BackendState {
  wasm: ExecuteResult;
  wasm_gc: ExecuteResult;
  js: ExecuteResult;
}

interface CBT {
  check: BackendState;
  build: BackendState;
  test: BackendState;
}

interface BuildState {
  source: number;
  cbts: CBT[];
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

  const renderBackendState = (backendState: BackendState) => (
    <>
      <td className={`py-2 px-4 ${getStatusStyle(backendState.wasm.status)} border-r`}>
        {getStatusText(backendState.wasm.status, backendState.wasm.elapsed)}
      </td>
      <td className={`py-2 px-4 ${getStatusStyle(backendState.wasm_gc.status)} border-r`}>
        {getStatusText(backendState.wasm_gc.status, backendState.wasm_gc.elapsed)}
      </td>
      <td className={`py-2 px-4 ${getStatusStyle(backendState.js.status)} border-r`}>
        {getStatusText(backendState.js.status, backendState.js.elapsed)}
      </td>
    </>
  );

  const renderTableRows = (
    stableData: BuildState[],
    bleedingData: BuildState[],
    sources: MooncakeSource[]
  ) => {
    return stableData.map((stableEntry, index) => {
      const source = sources[stableEntry.source];
      const isGit = "Git" in source;
      const versions = isGit ? source.Git.rev : source.MooncakesIO.version;
      const rowSpan = versions.length; // Number of versions determines the row span
  
      return versions.map((_, versionIndex) => {
        const bleedingEntry = bleedingData[index];
        const stableCBT = stableEntry.cbts[versionIndex];
        const bleedingCBT = bleedingEntry?.cbts[versionIndex];
  
        return (
          <tr key={`${index}-${versionIndex}`} className="border-b hover:bg-gray-50 text-sm">
            {/* Only display the source name in the first row */}
            {versionIndex === 0 && (
              <td className="py-2 px-4" rowSpan={rowSpan}>
                {isGit ? (
                  <>
                    <i className="fab fa-github text-gray-700 mr-2"></i>
                    <a
                      href={source.Git.url}
                      className="text-blue-600 hover:text-blue-800"
                      target="_blank"
                      rel="noopener noreferrer"
                    >
                      {source.Git.url.replace("https://github.com/", "")}
                    </a>
                  </>
                ) : (
                  <a
                    href={`https://mooncakes.io/docs/#/${source.MooncakesIO.name}/`}
                    className="text-blue-600 hover:text-blue-800"
                    target="_blank"
                    rel="noopener noreferrer"
                  >
                    {source.MooncakesIO.name}
                  </a>
                )}
              </td>
            )}
            {/* Display the version without a link if it is MooncakesIO */}
            <td className="py-2 px-4 text-gray-500">
              {isGit ? (
                <a
                  href={`${source.Git.url}/tree/${versions[versionIndex]}`}
                  className="text-blue-600 hover:text-blue-800"
                  target="_blank"
                  rel="noopener noreferrer"
                >
                  {versions[versionIndex]}
                </a>
              ) : (
                versions[versionIndex]
              )}
            </td>
  
            {/* Stable Data */}
            {stableCBT ? (
              <>
                {renderBackendState(stableCBT.check)}
                {renderBackendState(stableCBT.build)}
                {renderBackendState(stableCBT.test)}
              </>
            ) : (
              <td colSpan={9} className="py-2 px-4 text-center text-gray-500">
                No stable data available
              </td>
            )}
  
            {/* Bleeding Data */}
            {bleedingCBT ? (
              <>
                {renderBackendState(bleedingCBT.check)}
                {renderBackendState(bleedingCBT.build)}
                {renderBackendState(bleedingCBT.test)}
              </>
            ) : (
              <td colSpan={9} className="py-2 px-4 text-center text-gray-500">
                No bleeding data available
              </td>
            )}
          </tr>
        );
      });
    });
  };
  

  return (
    <div className="p-4 bg-gray-100 min-h-screen flex justify-center">
      <div className="w-full">
        <h1 className="text-2xl font-bold mb-4">Moon Build Dashboard</h1>
  
        {error ? (
          <p className="text-red-500 text-center">{error}</p>
        ) : data ? (
          <div className="overflow-x-auto">
            <table className="min-w-full table-auto bg-white shadow-md rounded-lg overflow-hidden">
              <thead>
                <tr className="bg-gray-200">
                  <th rowSpan={3} className="py-2 px-4 text-left w-1/4 border-r">Repository</th>
                  <th rowSpan={3} className="py-2 px-4 text-left w-1/4 border-r">Version</th>
                  <th colSpan={9} className="py-2 px-4 text-center bg-green-500 text-white border-r">
                    Stable Release
                    <div className="text-xs mt-1 font-normal">
                      {data.stable_toolchain_version.moon_version} / moonc {data.stable_toolchain_version.moonc_version}
                    </div>
                  </th>
                  <th
                    colSpan={9}
                    className="py-2 px-4 text-center bg-red-600 text-white relative overflow-hidden"
                  >
                    <span className="absolute inset-0 flex items-center justify-left text-6xl text-yellow-900 opacity-40">
                      ⚡️
                    </span>
                    Bleeding Edge Release
                    <div className="text-xs mt-1 font-normal">
                      {data.bleeding_toolchain_version.moon_version} / moonc {data.bleeding_toolchain_version.moonc_version}
                    </div>
                  </th>
                </tr>
                <tr className="bg-gray-100">
                  <th colSpan={3} className="py-1 px-4 text-center text-sm border-r">Check</th>
                  <th colSpan={3} className="py-1 px-4 text-center text-sm border-r">Build</th>
                  <th colSpan={3} className="py-1 px-4 text-center text-sm">Test</th>
                  <th colSpan={3} className="py-1 px-4 text-center text-sm border-r">Check</th>
                  <th colSpan={3} className="py-1 px-4 text-center text-sm border-r">Build</th>
                  <th colSpan={3} className="py-1 px-4 text-center text-sm">Test</th>
                </tr>
                <tr className="bg-gray-100">
                  <th className="py-1 px-4 text-left text-xs border-r">wasm</th>
                  <th className="py-1 px-4 text-left text-xs border-r">wasm gc</th>
                  <th className="py-1 px-4 text-left text-xs border-r">js</th>
                  <th className="py-1 px-4 text-left text-xs border-r">wasm</th>
                  <th className="py-1 px-4 text-left text-xs border-r">wasm gc</th>
                  <th className="py-1 px-4 text-left text-xs border-r">js</th>
                  <th className="py-1 px-4 text-left text-xs border-r">wasm</th>
                  <th className="py-1 px-4 text-left text-xs border-r">wasm gc</th>
                  <th className="py-1 px-4 text-left text-xs">js</th>
                  <th className="py-1 px-4 text-left text-xs border-r">wasm</th>
                  <th className="py-1 px-4 text-left text-xs border-r">wasm gc</th>
                  <th className="py-1 px-4 text-left text-xs border-r">js</th>
                  <th className="py-1 px-4 text-left text-xs border-r">wasm</th>
                  <th className="py-1 px-4 text-left text-xs border-r">wasm gc</th>
                  <th className="py-1 px-4 text-left text-xs border-r">js</th>
                  <th className="py-1 px-4 text-left text-xs border-r">wasm</th>
                  <th className="py-1 px-4 text-left text-xs border-r">wasm gc</th>
                  <th className="py-1 px-4 text-left text-xs">JS</th>
                </tr>
              </thead>
              <tbody>
                {renderTableRows(data.stable_release_data, data.bleeding_release_data, data.sources)}
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
