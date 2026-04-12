import type { GpuPool } from '../api/client';

interface Props {
  pool: GpuPool;
}

export default function GpuCapacityCard({ pool }: Props) {
  const pct = pool.total > 0 ? Math.round((pool.available / pool.total) * 100) : 0;
  const memGb = Math.round(pool.memory_mb / 1024);

  return (
    <div className="bg-gray-900 border border-gray-800 rounded-lg p-6">
      <div className="flex justify-between items-start mb-4">
        <div>
          <h3 className="text-lg font-semibold text-white uppercase">{pool.gpu_type}</h3>
          <p className="text-sm text-gray-400">{memGb} GB VRAM</p>
        </div>
        <span className={`text-2xl font-bold ${pct > 50 ? 'text-green-400' : pct > 20 ? 'text-yellow-400' : 'text-red-400'}`}>
          {pool.available}/{pool.total}
        </span>
      </div>
      <div className="w-full bg-gray-800 rounded-full h-2">
        <div
          className={`h-2 rounded-full ${pct > 50 ? 'bg-green-500' : pct > 20 ? 'bg-yellow-500' : 'bg-red-500'}`}
          style={{ width: `${pct}%` }}
        />
      </div>
      <p className="text-xs text-gray-500 mt-2">
        {pool.allocated} allocated across {pool.nodes.length} node{pool.nodes.length !== 1 ? 's' : ''}
      </p>
    </div>
  );
}
