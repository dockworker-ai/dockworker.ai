import { useStore } from '../store';
import './Settings.css';

export default function Settings() {
  const { user, quota } = useStore();

  return (
    <div className="settings">
      <h1>Account Settings</h1>

      <section>
        <h2>Profile</h2>
        <div className="setting-row">
          <label>Username</label>
          <input type="text" value={user?.username} disabled />
        </div>
        <div className="setting-row">
          <label>Email</label>
          <input type="email" value={user?.email} disabled />
        </div>
        <div className="setting-row">
          <label>Tier</label>
          <input type="text" value={user?.tier} disabled />
        </div>
      </section>

      <section>
        <h2>Quota</h2>
        <div className="setting-row">
          <label>Monthly Build Minutes</label>
          <input type="text" value={quota.monthlyMinutes} disabled />
        </div>
        <div className="setting-row">
          <label>Max Concurrent Builds</label>
          <input type="text" value={quota.maxConcurrent} disabled />
        </div>
      </section>

      <section>
        <h2>API Keys</h2>
        <p>Coming soon</p>
      </section>
    </div>
  );
}
