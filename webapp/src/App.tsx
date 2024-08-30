import React, { useState, useEffect } from 'react';

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

function transform_data(rows: Row[]): RepoEntry[] {
  const repoMap: { [key: string]: RepoEntry } = {};

  rows.forEach((row) => {
    const { repo, rev, command, status, elapsed, start_time } = row;
    const key = `${repo}@${rev}`;

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

  return Object.values(repoMap);
}

interface TransformedEntry {
  repo: string;
  rev: string;
  check: number | null;
  build: number | null;
  bundle: number | null;
  test: number | null;
  start_time: string;
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
        check: check ? check.status : null,
        build: build ? build.status : null,
        bundle: bundle ? bundle.status : null,
        test: test ? test.status : null,
        start_time: '',
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
  const [error, setError] = useState<string | null>(null);

  const fetchData = async () => {
    try {
      const parsedData = await get_data();
      const transformedData = transform_data(parsedData);
      const transformedData2 = transform_data2(transformedData);
      console.log(transformedData2);
      setData(transformedData2);
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

  // Helper function to determine background color based on status
  const getStatusColor = (status: number | null) => {
    if (status === null) return 'bg-gray-200'; // Default color for null
    return status === 0 ? 'bg-green-200' : 'bg-red-200';
  };

  return (
    <div className="p-8 bg-gray-100 min-h-screen">
      <h1 className="text-2xl font-bold mb-6 text-center">Moon Build Dashboard</h1>
      {error ? (
        <p className="text-red-500 text-center">{error}</p>
      ) : (
        <div className="overflow-x-auto">
          <table className="min-w-full bg-white shadow-md rounded-lg overflow-hidden">
            <thead className="bg-blue-500 text-white">
              <tr>
                <th className="py-3 px-6 text-left">Repository</th>
                <th className="py-3 px-6 text-left">Check</th>
                <th className="py-3 px-6 text-left">Build</th>
                <th className="py-3 px-6 text-left">Bundle</th>
                <th className="py-3 px-6 text-left">Test</th>
                <th className="py-3 px-6 text-left">Start Time</th>
              </tr>
            </thead>
            <tbody>
              {data.map((entry, index) => (
                <tr key={index} className="border-b hover:bg-gray-50">
                  <td className="py-3 px-6">
                    <a
                      href={entry.repo}
                      className="text-blue-600 hover:text-blue-800"
                      target="_blank"
                      rel="noopener noreferrer"
                    >
                      {entry.repo.replace('https://github.com/', '')}@{entry.rev}
                    </a>
                  </td>
                  <td className={`py-3 px-6 ${getStatusColor(entry.check)}`} />
                  <td className={`py-3 px-6 ${getStatusColor(entry.build)}`} />
                  <td className={`py-3 px-6 ${getStatusColor(entry.bundle)}`} />
                  <td className={`py-3 px-6 ${getStatusColor(entry.test)}`} />
                  <td className="py-3 px-6">{entry.start_time}</td>
                </tr>
              ))}
            </tbody>
          </table>
        </div>
      )}
    </div>
  );
};

export default App;
