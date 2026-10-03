import { useParams, useNavigate } from 'react-router-dom';
import { useEffect, useRef } from 'react';
import { useStore } from '../store';
import './BuildDetail.css';

export default function BuildDetail() {
  const { buildId } = useParams<{ buildId: string }>();
  const navigate = useNavigate();
  const { user } = useStore();
  const terminalRef = useRef<HTMLDivElement>(null);
  const [logs, setLogs] = React.useState<string>('');

  useEffect(() => {
    if (!buildId || !user?.token) return;

    const eventSource = new EventSource(`/api/builds/${buildId}/logs`);

    eventSource.addEventListener('log', (e) => {
      const data = JSON.parse(e.data);
      setLogs((prev) => prev + data.payload);
    });

    eventSource.addEventListener('done', () => {
      eventSource.close();
    });

    return () => eventSource.close();
  }, [buildId, user?.token]);

  return (
    <div className="build-detail">
      <button onClick={() => navigate('/dashboard')} className="back-button">
        ← Back
      </button>
      <h1>Build {buildId}</h1>
      <div className="terminal" ref={terminalRef}>
        <pre>{logs}</pre>
      </div>
    </div>
  );
}
