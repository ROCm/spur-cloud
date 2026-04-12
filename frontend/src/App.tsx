import { BrowserRouter, Routes, Route, Navigate } from 'react-router-dom';
import { useState, useEffect, createContext, useContext } from 'react';
import Login from './pages/Login';
import Dashboard from './pages/Dashboard';
import NewSession from './pages/NewSession';
import SessionDetail from './pages/SessionDetail';
import Settings from './pages/Settings';
import Billing from './pages/Billing';
import Navbar from './components/Navbar';

interface AuthContextType {
  token: string | null;
  user: { id: string; email: string; username: string; is_admin: boolean } | null;
  login: (token: string, user: AuthContextType['user']) => void;
  logout: () => void;
}

export const AuthContext = createContext<AuthContextType>({
  token: null,
  user: null,
  login: () => {},
  logout: () => {},
});

export function useAuth() {
  return useContext(AuthContext);
}

function ProtectedRoute({ children }: { children: React.ReactNode }) {
  const { token } = useAuth();
  if (!token) return <Navigate to="/login" />;
  return <>{children}</>;
}

export default function App() {
  const [token, setToken] = useState<string | null>(localStorage.getItem('token'));
  const [user, setUser] = useState<AuthContextType['user']>(() => {
    const saved = localStorage.getItem('user');
    return saved ? JSON.parse(saved) : null;
  });

  // Handle OAuth callback token from URL fragment
  useEffect(() => {
    const hash = window.location.hash;
    if (hash.includes('token=')) {
      const params = new URLSearchParams(hash.split('?')[1]);
      const callbackToken = params.get('token');
      if (callbackToken) {
        setToken(callbackToken);
        localStorage.setItem('token', callbackToken);
        // Decode JWT to get user info
        try {
          const payload = JSON.parse(atob(callbackToken.split('.')[1]));
          const u = { id: payload.sub, email: payload.email, username: payload.username, is_admin: payload.admin };
          setUser(u);
          localStorage.setItem('user', JSON.stringify(u));
        } catch { /* ignore */ }
        window.location.hash = '';
      }
    }
  }, []);

  const login = (newToken: string, newUser: AuthContextType['user']) => {
    setToken(newToken);
    setUser(newUser);
    localStorage.setItem('token', newToken);
    if (newUser) localStorage.setItem('user', JSON.stringify(newUser));
  };

  const logout = () => {
    setToken(null);
    setUser(null);
    localStorage.removeItem('token');
    localStorage.removeItem('user');
  };

  return (
    <AuthContext.Provider value={{ token, user, login, logout }}>
      <BrowserRouter>
        <div className="min-h-screen bg-gray-950 text-gray-100">
          {token && <Navbar />}
          <main className={token ? 'pt-16' : ''}>
            <Routes>
              <Route path="/login" element={<Login />} />
              <Route path="/" element={<ProtectedRoute><Dashboard /></ProtectedRoute>} />
              <Route path="/sessions/new" element={<ProtectedRoute><NewSession /></ProtectedRoute>} />
              <Route path="/sessions/:id" element={<ProtectedRoute><SessionDetail /></ProtectedRoute>} />
              <Route path="/settings" element={<ProtectedRoute><Settings /></ProtectedRoute>} />
              <Route path="/billing" element={<ProtectedRoute><Billing /></ProtectedRoute>} />
              <Route path="*" element={<Navigate to="/" />} />
            </Routes>
          </main>
        </div>
      </BrowserRouter>
    </AuthContext.Provider>
  );
}
