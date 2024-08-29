import React, { useState, useEffect } from 'react';

const App = () => {
  const [data, setData] = useState([]);
  const [error, setError] = useState(null);

  // Function to fetch and parse the JSONL file
  const fetchData = async () => {
    try {
      const response = await fetch('/data.jsonl'); // File is now in the public folder
      if (!response.ok) {
        throw new Error(`HTTP error! status: ${response.status}`);
      }
      const text = await response.text();
      const lines = text.split('\n').filter((line) => line.trim() !== '');
      const parsedData = lines.map((line) => JSON.parse(line));

      // Process data to get the latest status for each command by repo
      const transformedData = transformData(parsedData);
      setData(transformedData);
    } catch (err) {
      setError(err.message);
    }
  };

  // Helper function to transform data into the required format
  const transformData = (data) => {
    const repoMap = {};

    data.forEach((entry) => {
      const { repo, rev, command, status, elapsed, start_time } = entry;

      // Initialize the repo entry if it doesn't exist
      if (!repoMap[repo]) {
        repoMap[repo] = { repo, rev, Check: 'N/A', Build: 'N/A', Bundle: 'N/A', Test: 'N/A', start_time: 'N/A' };
      }

      // Update the latest command status
      repoMap[repo][command] = `${status} (${elapsed !== null ? `${elapsed} ms` : 'N/A'})`;

      // Keep track of the latest start time
      if (new Date(start_time) > new Date(repoMap[repo].start_time)) {
        repoMap[repo].start_time = start_time;
      }
    });

    return Object.values(repoMap);
  };

  // Fetch data on component mount
  useEffect(() => {
    fetchData();
  }, []);

  return (
    <div className="p-8 bg-gray-100 min-h-screen">
      <h1 className="text-2xl font-bold mb-6 text-center">Dashboard</h1>
      {error ? (
        <p className="text-red-500 text-center">{error}</p>
      ) : (
        <div className="overflow-x-auto">
          <table className="min-w-full bg-white shadow-md rounded-lg overflow-hidden">
            <thead className="bg-blue-500 text-white">
              <tr>
                <th className="py-3 px-6 text-left">Repo (Revision)</th>
                <th className="py-3 px-6 text-left">Check</th>
                <th className="py-3 px-6 text-left">Build</th>
                <th className="py-3 px-6 text-left">Bundle</th>
                <th className="py-3 px-6 text-left">Test</th>
                <th className="py-3 px-6 text-left">Latest Start Time</th>
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
                  <td className="py-3 px-6">{entry.Check}</td>
                  <td className="py-3 px-6">{entry.Build}</td>
                  <td className="py-3 px-6">{entry.Bundle}</td>
                  <td className="py-3 px-6">{entry.Test}</td>
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
