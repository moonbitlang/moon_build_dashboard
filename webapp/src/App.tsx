import { useState, useEffect } from 'react';

interface Row {
  repo: string;
  rev: string;
  command: string;
  moon_version: string;
  moonc_version: string;
  status: number;
  elapsed: number | null;
  start_time: string;
  run_id: string;
  run_number: string;
}

async function get_data(): Promise<Row[]> {
  const response = await fetch('/data.jsonl');
  const text = await response.text();
  const lines = text.split('\n').filter((line) => line.trim() !== '');
  const parsedData = lines.map((line) => JSON.parse(line));
  return parsedData;
}

interface RepoEntry {
  repo: string;
  rev: string;
  check: { status: number; elapsed: number | null; start_time: string } | null;
  build: { status: number; elapsed: number | null; start_time: string } | null;
  bundle: { status: number; elapsed: number | null; start_time: string } | null;
  test: { status: number; elapsed: number | null; start_time: string } | null;
}

function transform_data(rows: Row[]): { entries: RepoEntry[], moon_version: string, moonc_version: string, run_id: string } {
  const repoMap: { [key: string]: RepoEntry } = {};
  let moon_version = rows[0]?.moon_version || '';
  let moonc_version = rows[0]?.moonc_version || '';
  let run_id = rows[0]?.run_id || '';
  rows.forEach((row) => {
    const { repo, rev, command, status, elapsed, start_time } = row;
    const key = `${repo}@${rev}`;
    if (moon_version !== row.moon_version) {
      moon_version = row.moon_version;
    }
    if (moonc_version !== row.moonc_version) {
      moonc_version = row.moonc_version;
    }
    if (run_id !== row.run_id) {
      run_id = row.run_id;
    }
    if (!repoMap[key]) {
      repoMap[key] = {
        repo,
        rev,
        check: null,
        build: null,
        bundle: null,
        test: null,
      };
    }

    const commandEntry = { status, elapsed, start_time };

    if (command.toLowerCase() === 'check') {
      repoMap[key].check = commandEntry;
    } else if (command.toLowerCase() === 'build') {
      repoMap[key].build = commandEntry;
    } else if (command.toLowerCase() === 'bundle') {
      repoMap[key].bundle = commandEntry;
    } else if (command.toLowerCase() === 'test') {
      repoMap[key].test = commandEntry;
    }
  });

  return { entries: Object.values(repoMap), moon_version, moonc_version, run_id };
}

interface TransformedEntry {
  repo: string;
  rev: string;
  check: { status: number | null, elapsed: number | null };
  build: { status: number | null, elapsed: number | null };
  bundle: { status: number | null, elapsed: number | null };
  test: { status: number | null, elapsed: number | null };
  start_time: string;
  run_id: string;
}

interface RepoMap {
  [key: string]: TransformedEntry;
}

function transform_data2(entries: RepoEntry[]): TransformedEntry[] {
  const repoMap: RepoMap = {};

  entries.forEach((entry) => {
    const { repo, rev, check, build, bundle, test } = entry;
    const key = `${repo}@${rev}`;

    if (!repoMap[key]) {
      repoMap[key] = {
        repo,
        rev,
        check: { status: check ? check.status : null, elapsed: check ? check.elapsed : null },
        build: { status: build ? build.status : null, elapsed: build ? build.elapsed : null },
        bundle: { status: bundle ? bundle.status : null, elapsed: bundle ? bundle.elapsed : null },
        test: { status: test ? test.status : null, elapsed: test ? test.elapsed : null },
        start_time: '',
        run_id: '',
      };
    }

    const startTimes = [check?.start_time, build?.start_time, bundle?.start_time, test?.start_time].filter(Boolean);
    if (startTimes.length > 0) {
      repoMap[key].start_time = startTimes.sort((a, b) => new Date(b!).getTime() - new Date(a!).getTime())[0]!;
    }
  });

  return Object.values(repoMap);
}

const App = () => {
  const [data, setData] = useState<TransformedEntry[]>([]);
  const [moonVersion, setMoonVersion] = useState<string>('');
  const [mooncVersion, setMooncVersion] = useState<string>('');
  const [runId, setRunId] = useState<string>('');
  const [error, setError] = useState<string | null>(null);

  const fetchData = async () => {
    try {
      const parsedData = await get_data();
      const { entries, moon_version, moonc_version, run_id } = transform_data(parsedData);
      const transformedData2 = transform_data2(entries);
      setData(transformedData2);
      setMoonVersion(moon_version);
      setMooncVersion(moonc_version);
      setRunId(run_id);
    } catch (err) {
      if (err instanceof Error) {
        setError(err.message);
      } else {
        setError('Unknown error occurred');
      }
    }
  };

  useEffect(() => {
    fetchData();
  }, []);

  const getStatusStyle = (status: number | null): string => {
    if (status === null) return 'bg-gray-200 text-gray-800';
    return status === 0 ? 'bg-green-200 text-green-800' : 'bg-red-200 text-red-800';
  };

  const getStatusText = (status: number | null, elapsed: number | null): string => {
    if (status === null) return 'N/A';

    if (status === 0) {
      if (elapsed) {
        return `${elapsed} ms`;
      } else {
        return '-';
      }
    } else {
      if (elapsed) {
        return `${elapsed} ms`;
      } else {
        return '';
      }
    }
  };

  return (
    <div className="p-4 bg-gray-100 min-h-screen flex justify-center">
      <div className="max-w-[800px] w-full">
        <h1 className="text-2xl font-bold mb-2">Moon Build Dashboard</h1>
        <div className="mb-4">
          <p className="font-mono">moon version: {moonVersion}</p>
          <p className="font-mono">moonc version: {mooncVersion}</p>
          <p className="font-mono">
            GitHub Action:{' '}
            <a
              href={`https://github.com/moonbitlang/moon_build_dashboard/actions/runs/${runId}`}
              target="_blank"
              rel="noopener noreferrer"
            >
              {`https://github.com/moonbitlang/moon_build_dashboard/actions/runs/${runId}`}
            </a>
          </p>
        </div>
        {error ? (
          <p className="text-red-500 text-center">{error}</p>
        ) : (
          <div className="overflow-x-auto">
            <table className="min-w-full table-auto bg-white shadow-md rounded-lg overflow-hidden">
              <thead className="bg-blue-500 text-white">
                <tr>
                  <th className="py-2 px-4 text-left w-1/3">Repository</th>
                  <th className="py-2 px-4 text-left w-1/6">Check</th>
                  <th className="py-2 px-4 text-left w-1/6">Build</th>
                  <th className="py-2 px-4 text-left w-1/6">Test</th>
                  <th className="py-2 px-4 text-left w-1/6">Start Time</th>
                </tr>
              </thead>
              <tbody>
                {data.map((entry, index) => (
                  <tr key={index} className="border-b hover:bg-gray-50">
                    <td className="py-2 px-4">
                      <a
                        href={entry.repo}
                        className="text-blue-600 hover:text-blue-800"
                        target="_blank"
                        rel="noopener noreferrer"
                      >
                        {entry.repo.replace('https://github.com/', '')}
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
                    <td className="py-2 px-4 text-xs">{entry.start_time}</td>
                  </tr>
                ))}
              </tbody>
            </table>
          </div>
        )}
      </div>
    </div>
  );
};

export default App;
