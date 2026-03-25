import React from 'react';
import SecurityPanel from '../components/security/SecurityPanel';

const SecurityStatusPage: React.FC = () => {
  // Mock data for MVP
  const lastAuth = '2026-04-15 10:15:00';
  const lastEvent = 'Audit #001';
  const gateStatus = 'OK';

  return (
    <div style={{ padding: 16 }}>
      <h2>Security Status (MVP)</h2>
      <SecurityPanel lastAuth={lastAuth} lastEvent={lastEvent} gateStatus={gateStatus} />
    </div>
  );
};

export default SecurityStatusPage;
