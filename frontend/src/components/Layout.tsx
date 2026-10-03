import { Outlet, useNavigate } from 'react-router-dom';
import { useStore } from '../store';
import './Layout.css';

export default function Layout() {
  const navigate = useNavigate();
  const { user, logout } = useStore();

  const handleLogout = () => {
    logout();
    navigate('/login');
  };

  return (
    <div className="layout">
      <nav className="sidebar">
        <div className="logo">🐳 dockworker</div>
        <ul className="nav-links">
          <li><a href="/dashboard">Dashboard</a></li>
          <li><a href="/settings">Settings</a></li>
        </ul>
        <div className="user-info">
          <p>{user?.username}</p>
          <p className="tier">{user?.tier}</p>
          <button onClick={handleLogout}>Logout</button>
        </div>
      </nav>
      <main className="content">
        <Outlet />
      </main>
    </div>
  );
}
