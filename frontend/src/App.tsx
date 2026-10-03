import { Routes, Route, Navigate } from 'react-router-dom';
import { useStore } from './store';
import Layout from './components/Layout';
import Login from './pages/Login';
import Dashboard from './pages/Dashboard';
import BuildDetail from './pages/BuildDetail';
import Settings from './pages/Settings';

export default function App() {
  const { user } = useStore();

  return (
    <Routes>
      <Route path="/login" element={<Login />} />

      {user ? (
        <Route element={<Layout />}>
          <Route path="/dashboard" element={<Dashboard />} />
          <Route path="/builds/:buildId" element={<BuildDetail />} />
          <Route path="/settings" element={<Settings />} />
          <Route path="/" element={<Navigate to="/dashboard" />} />
        </Route>
      ) : (
        <Route path="/*" element={<Navigate to="/login" />} />
      )}
    </Routes>
  );
}
