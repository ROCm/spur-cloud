import { useState, useEffect } from 'react';
import { sessions, gpus, type Session, type GpuPool } from '../api/client';
import GpuCapacityCard from '../components/GpuCapacityCard';
import SessionTable from '../components/SessionTable';

export default function Dashboard() {
  const [sessionList, setSessionList] = useState<Session[]>([]);
  const [gpuPools, setGpuPools] = useState<GpuPool[]>([]);
  const [loading, setLoading] = useState(true);

  const refresh = async () => {
    try {
      const [s, g] = await Promise.all([sessions.list(), gpus.capacity()]);
      setSessionList(s);
      setGpuPools(g);
    } catch {
      // ignore
    } finally {
      setLoading(false);
    }
  };

  useEffect(() => {
    refresh();
    const interval = setInterval(refresh, 10000);
    return () => clearInterval(interval);
  }, []);

  if (loading) {
    return <div className="max-w-7xl mx-auto px-4 py-8 text-gray-400">Loading...</div>;
  }

  return (
    <div className="max-w-7xl mx-auto px-4 py-8">
      {/* GPU Capacity */}
      <h2 className="text-xl font-semibold text-white mb-4">GPU Capacity</h2>
      <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-4 mb-8">
        {gpuPools.length === 0 ? (
          <p className="text-gray-500 col-span-3">No GPU nodes registered</p>
        ) : (
          gpuPools.map(p => <GpuCapacityCard key={p.gpu_type} pool={p} />)
        )}
      </div>

      {/* Sessions */}
      <div className="flex items-center justify-between mb-4">
        <h2 className="text-xl font-semibold text-white">Sessions</h2>
        <a
          href="/sessions/new"
          className="px-4 py-2 bg-blue-600 hover:bg-blue-500 text-white rounded-lg text-sm font-medium transition"
        >
          Launch Session
        </a>
      </div>
      <div className="bg-gray-900 border border-gray-800 rounded-lg">
        <SessionTable sessions={sessionList} />
      </div>
    </div>
  );
}
