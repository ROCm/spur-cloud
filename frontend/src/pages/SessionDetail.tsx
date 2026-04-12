import { useState, useEffect } from 'react';
import { useParams, useNavigate } from 'react-router-dom';
import { sessions, terminalWsUrl, type Session } from '../api/client';
import Terminal from '../components/Terminal';

export default function SessionDetail() {
  const { id } = useParams<{ id: string }>();
  const navigate = useNavigate();
  const [session, setSession] = useState<Session | null>(null);
  const [showTerminal, setShowTerminal] = useState(false);
  const [deleting, setDeleting] = useState(false);

  useEffect(() => {
    if (!id) return;
    const refresh = () => {
      sessions.get(id).then(setSession).catch(() => navigate('/'));
    };
    refresh();
    const interval = setInterval(refresh, 5000);
    return () => clearInterval(interval);
  }, [id, navigate]);

  if (!session) {
    return <div className="max-w-4xl mx-auto px-4 py-8 text-gray-400">Loading...</div>;
  }

  const isRunning = session.state === 'running';
  const isTerminal = ['completed', 'failed', 'cancelled'].includes(session.state);

  const handleDelete = async () => {
    if (!confirm('Terminate this session?')) return;
    setDeleting(true);
    try {
      await sessions.delete(session.id);
      navigate('/');
    } catch {
      setDeleting(false);
    }
  };

  return (
    <div className="max-w-4xl mx-auto px-4 py-8">
      <div className="flex items-center justify-between mb-6">
        <div>
          <h1 className="text-2xl font-bold text-white">{session.name}</h1>
          <p className="text-gray-400 text-sm">Job #{session.spur_job_id || 'pending'}</p>
        </div>
        <div className="flex gap-3">
          {isRunning && !showTerminal && (
            <button
              onClick={() => setShowTerminal(true)}
              className="px-4 py-2 bg-green-600 hover:bg-green-500 text-white rounded-lg text-sm font-medium transition"
            >
              Open Terminal
            </button>
          )}
          {!isTerminal && (
            <button
              onClick={handleDelete}
              disabled={deleting}
              className="px-4 py-2 bg-red-600 hover:bg-red-500 disabled:bg-red-800 text-white rounded-lg text-sm font-medium transition"
            >
              {deleting ? 'Stopping...' : 'Terminate'}
            </button>
          )}
        </div>
      </div>

      {/* Session Info */}
      <div className="bg-gray-900 border border-gray-800 rounded-lg p-6 mb-6">
        <div className="grid grid-cols-2 md:grid-cols-4 gap-4">
          <InfoItem label="State" value={session.state} />
          <InfoItem label="GPU" value={`${session.gpu_count}x ${session.gpu_type}`} />
          <InfoItem label="Image" value={session.container_image} />
          <InfoItem label="Node" value={session.node_name || 'pending'} />
          <InfoItem label="Time Limit" value={`${session.time_limit_min} min`} />
          <InfoItem label="Created" value={new Date(session.created_at).toLocaleString()} />
          {session.started_at && (
            <InfoItem label="Started" value={new Date(session.started_at).toLocaleString()} />
          )}
          {session.ended_at && (
            <InfoItem label="Ended" value={new Date(session.ended_at).toLocaleString()} />
          )}
        </div>

        {/* SSH Info */}
        {session.ssh_enabled && session.ssh_port && (
          <div className="mt-4 p-4 bg-gray-800 rounded-lg">
            <h3 className="text-sm font-medium text-gray-300 mb-2">SSH Access</h3>
            <code className="text-green-400 text-sm">
              ssh -p {session.ssh_port} root@{session.ssh_host || session.node_name || '<node-ip>'}
            </code>
          </div>
        )}
      </div>

      {/* Terminal */}
      {showTerminal && isRunning && (
        <div className="mb-6">
          <div className="flex items-center justify-between mb-2">
            <h2 className="text-lg font-semibold text-white">Terminal</h2>
            <button
              onClick={() => setShowTerminal(false)}
              className="text-sm text-gray-400 hover:text-white"
            >
              Close
            </button>
          </div>
          <Terminal wsUrl={terminalWsUrl(session.id)} />
        </div>
      )}
    </div>
  );
}

function InfoItem({ label, value }: { label: string; value: string }) {
  return (
    <div>
      <p className="text-xs text-gray-500 uppercase">{label}</p>
      <p className="text-sm text-gray-200 truncate" title={value}>{value}</p>
    </div>
  );
}
