import React from 'react';

type SecurityPanelProps = {
  lastAuth?: string; // timestamp or label
  lastEvent?: string;
  gateStatus?: string; // e.g., 'OK' | 'Blocked'
};

const SecurityPanel: React.FC<SecurityPanelProps> = ({ lastAuth, lastEvent, gateStatus = 'OK' }) => {
  return (
    <div style={{ border: '1px solid #e2e8f0', borderRadius: 8, padding: 12, background: '#fff' }}>
      <div style={{ fontWeight: 700, marginBottom: 8 }}>Security</div>
      <div style={{ display: 'flex', gap: 12, flexDirection: 'row' }}>
        <div style={{ minWidth: 180 }}>
          <div style={{ fontSize: 12, color: '#666' }}>Last Auth</div>
          <div style={{ fontWeight: 600 }}>{lastAuth ?? 'not set'}</div>
        </div>
        <div style={{ minWidth: 180 }}>
          <div style={{ fontSize: 12, color: '#666' }}>Last Event</div>
          <div style={{ fontWeight: 600 }}>{lastEvent ?? 'none'}</div>
        </div>
        <div style={{ minWidth: 120 }}>
          <div style={{ fontSize: 12, color: '#666' }}>Gate</div>
          <div style={{ fontWeight: 700, color: gateStatus === 'OK' ? '#1a8cff' : '#e02424' }}>{gateStatus}</div>
        </div>
      </div>
    </div>
  );
};

export default SecurityPanel;
