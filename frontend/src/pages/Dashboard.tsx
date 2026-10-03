import { useEffect, useState } from 'react';
import { useStore } from '../store';
import axios from 'axios';
import './Dashboard.css';

export default function Dashboard() {
  const { user, builds, quota, setBuilds, setQuota } = useStore();
  const [loading, setLoading] = useState(true);

  useEffect(() => {
    const fetchData = async () => {
      try {
        const [buildsRes, quotaRes] = await Promise.all([
          axios.get('/api/builds', {
            headers: { Authorization: `Bearer ${user?.token}` },
          }),
          axios.get('/api/user/quota', {
            headers: { Authorization: `Bearer ${user?.token}` },
          }),
        ]);
        setBuilds(buildsRes.data.builds || []);
        setQuota(quotaRes.data);
      } catch (error) {
        console.error('Failed to fetch data:', error);
      } finally {
        setLoading(false);
      }
    };
    fetchData();
  }, [user?.token]);

  return (
    <div className="dashboard">
      <h1>Builds</h1>

      <div className="quota-card">
        <h3>Monthly Quota</h3>
        <div className="quota-bar">
          <div
            className="quota-used"
            style={{
              width: `${((quota.monthlyMinutes - quota.remainingMinutes) / quota.monthlyMinutes) * 100}%`
            }}
          />
        </div>
        <p>{quota.remainingMinutes} / {quota.monthlyMinutes} minutes remaining</p>
      </div>

      <div className="builds-list">
        {loading ? (
          <p>Loading builds...</p>
        ) : builds.length === 0 ? (
          <p>No builds yet. Create your first build!</p>
        ) : (
          <table>
            <thead>
              <tr>
                <th>ID</th>
                <th>Image</th>
                <th>Status</th>
                <th>Created</th>
                <th>Duration</th>
              </tr>
            </thead>
            <tbody>
              {builds.map((build) => (
                <tr key={build.id}>
                  <td><a href={`/builds/${build.id}`}>{build.id}</a></td>
                  <td>{build.imageName}</td>
                  <td><span className={`status ${build.status.toLowerCase()}`}>{build.status}</span></td>
                  <td>{new Date(build.createdAt).toLocaleString()}</td>
                  <td>{build.duration ? `${build.duration}s` : '-'}</td>
                </tr>
              ))}
            </tbody>
          </table>
        )}
      </div>
    </div>
  );
}
