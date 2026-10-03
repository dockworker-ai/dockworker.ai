import { useNavigate } from 'react-router-dom';
import { useStore } from '../store';
import axios from 'axios';
import './Login.css';

export default function Login() {
  const navigate = useNavigate();
  const { setUser } = useStore();

  const handleGitHubLogin = async () => {
    try {
      // Redirect to GitHub OAuth
      const clientId = import.meta.env.VITE_GITHUB_CLIENT_ID;
      const redirectUri = `${window.location.origin}/auth/github/callback`;
      const url = `https://github.com/login/oauth/authorize?client_id=${clientId}&redirect_uri=${redirectUri}&scope=read:user,user:email`;
      window.location.href = url;
    } catch (error) {
      console.error('Login failed:', error);
    }
  };

  return (
    <div className="login-container">
      <div className="login-box">
        <h1>🐳 dockworker</h1>
        <p>Fast container builds for everyone</p>
        <button onClick={handleGitHubLogin} className="github-button">
          Sign in with GitHub
        </button>
        <p className="footer">Free tier: 200 min/month • Always free for open-source</p>
      </div>
    </div>
  );
}
