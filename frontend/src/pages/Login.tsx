import { useState, useEffect } from 'react';
import { useNavigate } from 'react-router-dom';
import { useAuth } from '../App';
import { auth, type Provider } from '../api/client';

export default function Login() {
  const { login, token } = useAuth();
  const navigate = useNavigate();
  const [mode, setMode] = useState<'login' | 'register'>('login');
  const [email, setEmail] = useState('');
  const [username, setUsername] = useState('');
  const [password, setPassword] = useState('');
  const [error, setError] = useState('');
  const [loading, setLoading] = useState(false);
  const [providers, setProviders] = useState<Provider[]>([]);

  useEffect(() => {
    if (token) navigate('/');
  }, [token, navigate]);

  useEffect(() => {
    auth.providers().then(r => setProviders(r.providers)).catch(() => {});
  }, []);

  const handleSubmit = async (e: React.FormEvent) => {
    e.preventDefault();
    setError('');
    setLoading(true);
    try {
      const resp = mode === 'login'
        ? await auth.login(email, password)
        : await auth.register(email, username, password);
      login(resp.token, resp.user);
      navigate('/');
    } catch (err: unknown) {
      setError(err instanceof Error ? err.message : 'Authentication failed');
    } finally {
      setLoading(false);
    }
  };

  return (
    <div className="min-h-screen flex items-center justify-center px-4">
      <div className="w-full max-w-md">
        <div className="text-center mb-8">
          <h1 className="text-4xl font-bold text-white mb-2">Spur Cloud</h1>
          <p className="text-gray-400">GPU as a Service</p>
        </div>

        <div className="bg-gray-900 border border-gray-800 rounded-xl p-8">
          {/* OAuth Providers */}
          {providers.filter(p => p.name !== 'local' && p.enabled).length > 0 && (
            <div className="space-y-3 mb-6">
              {providers.filter(p => p.name !== 'local' && p.enabled).map(p => (
                <a
                  key={p.name}
                  href={p.authorize_url}
                  className="w-full flex items-center justify-center gap-2 py-2.5 px-4 bg-gray-800 hover:bg-gray-700 border border-gray-700 rounded-lg text-white transition"
                >
                  {p.name === 'github' && (
                    <svg className="w-5 h-5" viewBox="0 0 24 24" fill="currentColor">
                      <path d="M12 0C5.37 0 0 5.37 0 12c0 5.31 3.435 9.795 8.205 11.385.6.105.825-.255.825-.57 0-.285-.015-1.23-.015-2.235-3.015.555-3.795-.735-4.035-1.41-.135-.345-.72-1.41-1.23-1.695-.42-.225-1.02-.78-.015-.795.945-.015 1.62.87 1.845 1.23 1.08 1.815 2.805 1.305 3.495.99.105-.78.42-1.305.765-1.605-2.67-.3-5.46-1.335-5.46-5.925 0-1.305.465-2.385 1.23-3.225-.12-.3-.54-1.53.12-3.18 0 0 1.005-.315 3.3 1.23.96-.27 1.98-.405 3-.405s2.04.135 3 .405c2.295-1.56 3.3-1.23 3.3-1.23.66 1.65.24 2.88.12 3.18.765.84 1.23 1.905 1.23 3.225 0 4.605-2.805 5.625-5.475 5.925.435.375.81 1.095.81 2.22 0 1.605-.015 2.895-.015 3.3 0 .315.225.69.825.57A12.02 12.02 0 0024 12c0-6.63-5.37-12-12-12z"/>
                    </svg>
                  )}
                  Continue with {p.name.charAt(0).toUpperCase() + p.name.slice(1)}
                </a>
              ))}
              <div className="relative my-4">
                <div className="absolute inset-0 flex items-center">
                  <div className="w-full border-t border-gray-700"></div>
                </div>
                <div className="relative flex justify-center text-sm">
                  <span className="bg-gray-900 px-4 text-gray-500">or</span>
                </div>
              </div>
            </div>
          )}

          {/* Local Auth Form */}
          <form onSubmit={handleSubmit} className="space-y-4">
            <div>
              <label className="block text-sm text-gray-400 mb-1">Email</label>
              <input
                type="email"
                value={email}
                onChange={e => setEmail(e.target.value)}
                className="w-full px-3 py-2 bg-gray-800 border border-gray-700 rounded-lg text-white focus:border-blue-500 focus:outline-none"
                required
              />
            </div>
            {mode === 'register' && (
              <div>
                <label className="block text-sm text-gray-400 mb-1">Username</label>
                <input
                  type="text"
                  value={username}
                  onChange={e => setUsername(e.target.value)}
                  className="w-full px-3 py-2 bg-gray-800 border border-gray-700 rounded-lg text-white focus:border-blue-500 focus:outline-none"
                  required
                />
              </div>
            )}
            <div>
              <label className="block text-sm text-gray-400 mb-1">Password</label>
              <input
                type="password"
                value={password}
                onChange={e => setPassword(e.target.value)}
                className="w-full px-3 py-2 bg-gray-800 border border-gray-700 rounded-lg text-white focus:border-blue-500 focus:outline-none"
                required
              />
            </div>
            {error && <p className="text-red-400 text-sm">{error}</p>}
            <button
              type="submit"
              disabled={loading}
              className="w-full py-2.5 bg-blue-600 hover:bg-blue-500 disabled:bg-blue-800 text-white rounded-lg font-medium transition"
            >
              {loading ? '...' : mode === 'login' ? 'Sign In' : 'Create Account'}
            </button>
          </form>

          <p className="text-center text-sm text-gray-500 mt-4">
            {mode === 'login' ? (
              <>No account? <button onClick={() => setMode('register')} className="text-blue-400 hover:underline">Register</button></>
            ) : (
              <>Have an account? <button onClick={() => setMode('login')} className="text-blue-400 hover:underline">Sign in</button></>
            )}
          </p>
        </div>
      </div>
    </div>
  );
}
