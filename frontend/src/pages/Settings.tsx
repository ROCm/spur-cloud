import { useState, useEffect } from 'react';
import { sshKeys, type SshKey } from '../api/client';

export default function Settings() {
  const [keys, setKeys] = useState<SshKey[]>([]);
  const [name, setName] = useState('');
  const [publicKey, setPublicKey] = useState('');
  const [error, setError] = useState('');
  const [adding, setAdding] = useState(false);

  const refresh = () => {
    sshKeys.list().then(setKeys).catch(() => {});
  };

  useEffect(refresh, []);

  const handleAdd = async (e: React.FormEvent) => {
    e.preventDefault();
    setError('');
    setAdding(true);
    try {
      await sshKeys.add(name, publicKey);
      setName('');
      setPublicKey('');
      refresh();
    } catch (err: unknown) {
      setError(err instanceof Error ? err.message : 'Failed to add key');
    } finally {
      setAdding(false);
    }
  };

  const handleDelete = async (id: string) => {
    if (!confirm('Delete this SSH key?')) return;
    try {
      await sshKeys.delete(id);
      refresh();
    } catch {
      // ignore
    }
  };

  return (
    <div className="max-w-3xl mx-auto px-4 py-8">
      <h1 className="text-2xl font-bold text-white mb-6">Settings</h1>

      {/* SSH Keys */}
      <div className="bg-gray-900 border border-gray-800 rounded-xl p-6 mb-6">
        <h2 className="text-lg font-semibold text-white mb-4">SSH Keys</h2>
        <p className="text-sm text-gray-400 mb-4">
          Add your SSH public keys to enable SSH access to GPU sessions.
        </p>

        {/* Key List */}
        {keys.length > 0 && (
          <div className="space-y-3 mb-6">
            {keys.map(key => (
              <div key={key.id} className="flex items-center justify-between p-3 bg-gray-800 rounded-lg">
                <div>
                  <p className="text-sm font-medium text-white">{key.name}</p>
                  <p className="text-xs text-gray-500 font-mono">{key.fingerprint}</p>
                </div>
                <button
                  onClick={() => handleDelete(key.id)}
                  className="text-sm text-red-400 hover:text-red-300"
                >
                  Delete
                </button>
              </div>
            ))}
          </div>
        )}

        {/* Add Key Form */}
        <form onSubmit={handleAdd} className="space-y-3">
          <input
            type="text"
            value={name}
            onChange={e => setName(e.target.value)}
            placeholder="Key name (e.g., Work Laptop)"
            className="w-full px-3 py-2 bg-gray-800 border border-gray-700 rounded-lg text-white focus:border-blue-500 focus:outline-none text-sm"
            required
          />
          <textarea
            value={publicKey}
            onChange={e => setPublicKey(e.target.value)}
            placeholder="ssh-ed25519 AAAA... or ssh-rsa AAAA..."
            rows={3}
            className="w-full px-3 py-2 bg-gray-800 border border-gray-700 rounded-lg text-white focus:border-blue-500 focus:outline-none text-sm font-mono"
            required
          />
          {error && <p className="text-red-400 text-sm">{error}</p>}
          <button
            type="submit"
            disabled={adding}
            className="px-4 py-2 bg-blue-600 hover:bg-blue-500 disabled:bg-blue-800 text-white rounded-lg text-sm font-medium transition"
          >
            {adding ? 'Adding...' : 'Add Key'}
          </button>
        </form>
      </div>
    </div>
  );
}
