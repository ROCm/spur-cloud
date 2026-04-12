import { Link, useNavigate } from 'react-router-dom';
import { useAuth } from '../App';

export default function Navbar() {
  const { user, logout } = useAuth();
  const navigate = useNavigate();

  return (
    <nav className="fixed top-0 left-0 right-0 bg-gray-900 border-b border-gray-800 z-50">
      <div className="max-w-7xl mx-auto px-4 flex items-center justify-between h-16">
        <div className="flex items-center gap-8">
          <Link to="/" className="text-xl font-bold text-white">
            Spur Cloud
          </Link>
          <div className="flex gap-4">
            <Link to="/" className="text-gray-300 hover:text-white transition">
              Dashboard
            </Link>
            <Link to="/sessions/new" className="text-gray-300 hover:text-white transition">
              Launch Session
            </Link>
            <Link to="/billing" className="text-gray-300 hover:text-white transition">
              Billing
            </Link>
            <Link to="/settings" className="text-gray-300 hover:text-white transition">
              Settings
            </Link>
          </div>
        </div>
        <div className="flex items-center gap-4">
          <span className="text-gray-400 text-sm">{user?.username}</span>
          <button
            onClick={() => { logout(); navigate('/login'); }}
            className="text-sm text-gray-400 hover:text-white transition"
          >
            Logout
          </button>
        </div>
      </div>
    </nav>
  );
}
