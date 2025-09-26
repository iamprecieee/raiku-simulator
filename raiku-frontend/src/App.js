import React, { useState, useEffect, useCallback, useRef } from 'react';
import { Zap, ChevronDown, ChevronLeft, ChevronRight, Users, User } from 'lucide-react';

const API_BASE = process.env.REACT_APP_API_BASE || 'http://localhost:8080';

const RaikuSimulator = () => {
  const [sessionId, setSessionId] = useState(null);
  const [currentSlot, setCurrentSlot] = useState(0);
  const [slots, setSlots] = useState([]);
  const [jitAuctions, setJitAuctions] = useState([]);
  const [aotAuctions, setAotAuctions] = useState([]);
  const [transactions, setTransactions] = useState([]);
  const [showAllTransactions, setShowAllTransactions] = useState(false);
  const [currentPage, setCurrentPage] = useState(1);
  const [pagination, setPagination] = useState({
    current_page: 1,
    total_pages: 1,
    page_size: 20,
    total_count: 0,
    has_next: false,
    has_prev: false
  });
  const [activeTab, setActiveTab] = useState('marketplace');
  const [notifications, setNotifications] = useState([]);
  const [isMobile, setIsMobile] = useState(false);
  const [dropdownOpen, setDropdownOpen] = useState(false);
  const [connectionStatus, setConnectionStatus] = useState('disconnected');
  const [stats, setStats] = useState({
    active_jit_auctions: 0,
    active_aot_auctions: 0,
    total_transactions: 0
  });
  const eventSourceRef = useRef(null);
  const notificationIdRef = useRef(0);

  useEffect(() => {
    const checkMobile = () => {
      setIsMobile(window.innerWidth <= 768);
    };
    
    checkMobile();
    window.addEventListener('resize', checkMobile);
    return () => window.removeEventListener('resize', checkMobile);
  }, []);

  const addNotification = useCallback((message, type = 'info') => {
    const id = ++notificationIdRef.current;
    setNotifications(prev => [...prev, { id, message, type }]);
    setTimeout(() => {
      setNotifications(prev => prev.filter(n => n.id !== id));
    }, 3000);
  }, []);
  
  const fetchJitAuctions = useCallback(async () => {
    try {
      const response = await fetch(`${API_BASE}/auctions/jit`);
      const data = await response.json();
      setJitAuctions(data.auctions || []);
    } catch (error) {
      console.error('Failed to fetch JIT auctions:', error);
    }
  }, []);

  const fetchAotAuctions = useCallback(async () => {
    try {
      const response = await fetch(`${API_BASE}/auctions/aot`);
      const data = await response.json();
      setAotAuctions(data.auctions || []);
    } catch (error) {
      console.error('Failed to fetch AOT auctions:', error);
    }
  }, []);

  const fetchTransactions = useCallback(async (page = 1) => {
    if (!sessionId) return;

    try {
      let url;
      if (showAllTransactions) {
        url = `${API_BASE}/transactions/all?page=${page}&limit=20`;
      } else {
        url = `${API_BASE}/transactions?session_id=${sessionId}&page=${page}&limit=20`;
      }
      
      const response = await fetch(url);
      const data = await response.json();
      
      setTransactions(data.transactions || []);
      setPagination(data.pagination);
      setCurrentPage(page);
    } catch (error) {
      console.error('Failed to fetch transactions:', error);
    }
  }, [sessionId, showAllTransactions]);
  
  const handleEvent = useCallback((event) => {
    switch (event.type) {
      case 'SlotAdvanced':
        setCurrentSlot(event.current_slot);
        break;
        
      case 'SlotsUpdated':
        setSlots(event.slots);
        break;
        
      case 'JitAuctionStarted':
        fetchJitAuctions();
        addNotification(`JIT auction started for slot ${event.slot_number}`, 'info');
        break;
        
      case 'AotAuctionStarted':
        fetchAotAuctions();
        addNotification(`AOT auction started for slot ${event.slot_number}`, 'info');
        break;
        
      case 'JitBidSubmitted':
        fetchJitAuctions();
        addNotification(`JIT bid submitted: ${event.amount} SOL`, 'success');
        break;
        
      case 'AotBidSubmitted':
        fetchAotAuctions();
        addNotification(`AOT bid submitted: ${event.amount} SOL`, 'success');
        break;
        
      case 'JitAuctionResolved':
        fetchJitAuctions();
        addNotification(`JIT auction won! Slot ${event.slot_number}: ${event.winning_bid} SOL`, 'success');
        break;
        
      case 'AotAuctionResolved':
        fetchAotAuctions();
        addNotification(`AOT auction won! Slot ${event.slot_number}: ${event.winning_bid} SOL`, 'success');
        break;
        
      case 'TransactionUpdated':
        if (sessionId && event.transaction.sender === sessionId) {
          fetchTransactions();
        }
        break;
        
      case 'MarketplaceStats':
        setStats({
          active_jit_auctions: event.active_jit_auctions,
          active_aot_auctions: event.active_aot_auctions,
          total_transactions: event.total_transactions
        });
        setCurrentSlot(event.current_slot);
        break;
        
      default:
        break;
    }
  }, [sessionId, addNotification, fetchAotAuctions, fetchJitAuctions]);
  
  const connectEventSource = useCallback(() => {
    if (eventSourceRef.current) {
      eventSourceRef.current.close();
    }

    const eventSource = new EventSource(`${API_BASE}/events`);
    eventSourceRef.current = eventSource;

    eventSource.onopen = () => {
      setConnectionStatus('connected');
      addNotification('Real-time connection established!', 'success');
    };

    eventSource.onmessage = (event) => {
      try {
        const data = JSON.parse(event.data);
        handleEvent(data);
      } catch (error) {
        console.error('Failed to parse event data:', error);
      }
    };

    eventSource.onerror = () => {
      setConnectionStatus('error');
      setTimeout(() => {
        if (sessionId) {
          connectEventSource();
        }
      }, 3000);
    };

    return eventSource;
  }, [sessionId, addNotification, handleEvent]);


  const createSession = useCallback(async () => {
    try {
      const existingSessionId = localStorage.getItem('raiku_session_id');
      
      const response = await fetch(`${API_BASE}/sessions`, { 
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({ session_id: existingSessionId })
      });
      
      const data = await response.json();
      setSessionId(data.session_id);
      
      localStorage.setItem('raiku_session_id', data.session_id);
      
      if (data.status === 'validated') {
        addNotification('Session restored successfully!', 'success');
      } else {
        addNotification('New session created successfully!', 'success');
      }
    } catch (error) {
      localStorage.removeItem('raiku_session_id');
      addNotification('Failed to create session. Please refresh the page.', 'error');
      console.error('Session creation error:', error);
    }
  }, [addNotification]);

  const fetchInitialData = useCallback(async () => {
    if (!sessionId) return;

    try {
      const [statusRes, slotsRes, jitRes, aotRes, txRes] = await Promise.all([
        fetch(`${API_BASE}/marketplace/status`),
        fetch(`${API_BASE}/marketplace/slots?session_id=${sessionId}`),
        fetch(`${API_BASE}/auctions/jit`),
        fetch(`${API_BASE}/auctions/aot`),
        fetch(`${API_BASE}/transactions?session_id=${sessionId}`)
      ]);

      const status = await statusRes.json();
      const slotsData = await slotsRes.json();
      const jitData = await jitRes.json();
      const aotData = await aotRes.json();
      const txData = await txRes.json();

      setCurrentSlot(status.current_slot);
      setSlots(slotsData.slots || []);
      setJitAuctions(jitData.auctions || []);
      setAotAuctions(aotData.auctions || []);
      setTransactions(txData.transactions || []);
    } catch (err) {
      console.error('Initial fetch error:', err);
    }
  }, [sessionId]);


  useEffect(() => {
    createSession();
  }, [createSession]);
    
  useEffect(() => {
    if (sessionId) {
      fetchInitialData();
      connectEventSource();
    }
    
    return () => {
      if (eventSourceRef.current) {
        eventSourceRef.current.close();
      }
    };
    
  }, [sessionId, fetchInitialData, connectEventSource]);

  useEffect(() => {
    if (activeTab === 'transactions' && sessionId) {
      fetchTransactions(1);
    }
  }, [activeTab, sessionId, showAllTransactions, fetchTransactions]);

  const submitJitBid = async () => {
    const bidAmount = parseFloat(prompt('Enter JIT bid amount (SOL):') || '0');
    if (bidAmount <= 0) return;

    try {
      const res = await fetch(`${API_BASE}/transactions/jit`, {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({
          session_id: sessionId,
          bid_amount: bidAmount,
          compute_units: 200000,
          data: 'JIT transaction'
        })
      });

      if (!res.ok) {
        throw new Error(`HTTP error! status: ${res.status}`);
      }
    } catch (err) {
      addNotification('JIT bid failed', 'error');
    }
  };

  const submitAotBid = async (slotNumber) => {
    const bidAmount = parseFloat(prompt('Enter AOT bid amount (SOL):') || '0');
    if (bidAmount <= 0) return;

    try {
      const res = await fetch(`${API_BASE}/transactions/aot`, {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({
          session_id: sessionId,
          slot_number: slotNumber,
          bid_amount: bidAmount,
          compute_units: 200000,
          data: 'AOT transaction'
        })
      });

      if (!res.ok) {
        throw new Error(`HTTP error! status: ${res.status}`);
      }
    } catch (err) {
      addNotification('AOT bid failed', 'error');
    }
  };

  const getSlotStyle = (state) => {
    let backgroundColor = '#666';
    if (typeof state === 'string') {
      if (state === 'Available') backgroundColor = '#75bd4f';
      if (state === 'Expired') backgroundColor = '#666';
    } else {
      if (state?.JiTAuction) backgroundColor = '#297cb3';
      if (state?.AoTAuction) backgroundColor = 'rgb(169, 56, 56)';
      if (state?.Reserved) backgroundColor = '#d97706';
      if (state?.Filled) backgroundColor = '#dc2626';
    }
    return { backgroundColor };
  };

  const getStateName = (state) => {
    if (typeof state === 'string') return state;
    if (state?.JiTAuction) return 'JIT Auction';
    if (state?.AoTAuction) return 'AOT Auction';
    if (state?.Reserved) return 'Reserved';
    if (state?.Filled) return 'Filled';
    return 'Unknown';
  };

  const getTransactionStatus = (status) => {
    if (typeof status === 'string') return status;
    if (status?.Included) return 'Included';
    if (status?.AuctionWon) return 'Auction Won';
    if (status?.Failed) return 'Failed';
    return 'Pending';
  };

  const getTransactionType = (type) => {
    if (typeof type === 'string') return type;
    if (type?.JiT) return 'JIT';
    if (type?.AoT) return `AOT (Slot ${type.AoT.reserved_slot})`;
    return 'Standard';
  };

  const handlePageChange = (newPage) => {
    if (newPage >= 1 && newPage <= pagination.total_pages) {
      fetchTransactions(newPage);
    }
  };
    
  const toggleTransactionView = () => {
    setShowAllTransactions(!showAllTransactions);
    setCurrentPage(1);
  };

  if (!sessionId) {
    return (
      <div style={{
        minHeight: '100vh',
        backgroundColor: 'rgb(236, 236, 222)',
        fontFamily: 'monospace',
        display: 'flex',
        alignItems: 'center',
        justifyContent: 'center',
        flexDirection: 'column',
        gap: '34px',
        padding: '20px'
      }}>
        <div style={{
          animation: 'spin 1s linear infinite',
          fontSize: isMobile ? '55px' : '68px'
        }}>
          <Zap size={isMobile ? 85 : 103} style={{ color: 'rgb(169, 56, 56)' }} />
        </div>
        <h2 style={{
          color: 'rgb(169, 56, 56)',
          fontWeight: 'bolder',
          textTransform: 'uppercase',
          margin: '0',
          fontSize: isMobile ? '30px' : '38px',
          textAlign: 'center'
        }}>
          Creating Your Raiku Session...
        </h2>
        <style>{`
          @keyframes spin {
            from { transform: rotate(0deg); }
            to { transform: rotate(360deg); }
          }
        `}</style>
      </div>
    );
  }

  const tabs = [
    { id: 'marketplace', name: 'Marketplace' },
    { id: 'auctions', name: 'Auctions' },
    { id: 'transactions', name: 'Transactions' },
  ];

  const handleTabChange = (tabId) => {
    setActiveTab(tabId);
    setDropdownOpen(false);
  };

  const getConnectionStatusColor = () => {
    switch (connectionStatus) {
      case 'connected': return '#75bd4f';
      case 'error': return 'rgb(169, 56, 56)';
      default: return '#666';
    }
  };

  return (
    <div style={{
      minHeight: '100vh',
      backgroundColor: 'rgb(236, 236, 222)',
      fontFamily: 'monospace',
      padding: isMobile ? '12px' : '17px',
      animation: 'fadeIn 0.5s ease-in-out'
    }}>
      <div style={{
        position: 'fixed',
        top: isMobile ? '12px' : '34px',
        right: isMobile ? '12px' : '34px',
        zIndex: 50,
        display: 'flex',
        flexDirection: 'column',
        gap: isMobile ? '12px' : '17px',
        maxWidth: isMobile ? '280px' : '350px'
      }}>
        {notifications.map(notification => (
          <div
            key={notification.id}
            style={{
              padding: isMobile ? '12px 17px' : '17px 26px',
              borderRadius: '5px',
              boxShadow: '0 4px 7px rgba(40, 40, 40, 1)',
              backgroundColor: notification.type === 'success' ? '#75bd4f' : 
                             notification.type === 'error' ? 'rgb(169, 56, 56)' : '#297cb3',
              color: 'rgb(252, 217, 157)',
              fontWeight: 'bold',
              textTransform: 'uppercase',
              animation: 'bubbleAppear 0.5s ease-in-out',
              fontSize: isMobile ? '15px' : '19px',
              wordBreak: 'break-word'
            }}
          >
            {notification.message}
          </div>
        ))}
      </div>

      <header style={{
        backgroundColor: 'rgb(236, 236, 222)',
        borderRadius: '5px',
        boxShadow: '0 4px 7px rgba(40, 40, 40, 1)',
        textAlign: 'center',
        textTransform: 'uppercase',
        width: '100%',
        transition: 'transform 1s ease',
        marginBottom: isMobile ? '24px' : '34px',
        animation: 'fadeIn 0.5s ease-in-out'
      }}
      onMouseEnter={(e) => e.target.style.transform = 'scale(1.05)'}
      onMouseLeave={(e) => e.target.style.transform = 'scale(1)'}
      >
        <h1 style={{
          animation: 'bubbleAppear 1s ease-in-out',
          border: '5px solid rgb(169, 56, 56)',
          borderRadius: '5px',
          boxShadow: '0 4px 7px rgba(40, 40, 40, 1)',
          color: 'rgb(169, 56, 56)',
          fontWeight: 'bolder',
          padding: isMobile ? '17px' : '26px',
          margin: '0',
          display: 'flex',
          alignItems: 'center',
          justifyContent: 'center',
          gap: isMobile ? '12px' : '17px',
          flexWrap: 'wrap',
          fontSize: isMobile ? '30px' : '40px'
        }}>
          <Zap size={isMobile ? 50 : 68} />
          Raiku Simulator
          <div style={{
            width: '16px',
            height: '16px',
            borderRadius: '50%',
            backgroundColor: getConnectionStatusColor(),
            marginLeft: '8px'
          }} />
        </h1>
      </header>

      <div style={{
        alignContent: 'center',
        backgroundColor: '#75bd4f',
        animation: 'bubbleAppear 0.5s ease-in-out',
        boxShadow: '0 4px 7px rgba(40, 40, 40, 1)',
        minHeight: isMobile ? '80px' : '102px',
        marginBottom: isMobile ? '24px' : '34px',
        textAlign: 'center',
        borderRadius: '5px',
        display: 'flex',
        alignItems: 'center',
        justifyContent: 'space-between',
        padding: isMobile ? '12px 17px' : '0 34px',
        flexDirection: isMobile ? 'column' : 'row',
        gap: isMobile ? '12px' : '34px'
      }}>
        <div style={{ 
          display: 'flex', 
          gap: isMobile ? '17px' : '34px',
          flexDirection: isMobile ? 'column' : 'row',
          alignItems: 'center',
          textAlign: 'center'
        }}>
          <span style={{ 
            fontWeight: 'bolder', 
            textTransform: 'uppercase', 
            color: 'rgb(252, 217, 157)',
            fontSize: isMobile ? '18px' : '23px'
          }}>
            Current Slot: {currentSlot}
          </span>
          <span style={{ 
            fontWeight: 'bolder', 
            textTransform: 'uppercase', 
            color: 'rgb(252, 217, 157)',
            fontSize: isMobile ? '18px' : '23px'
          }}>
            Auctions: {stats.active_jit_auctions + stats.active_aot_auctions}
          </span>
          <span style={{ 
            fontWeight: 'bolder', 
            textTransform: 'uppercase', 
            color: 'rgb(252, 217, 157)',
            fontSize: isMobile ? '18px' : '23px'
          }}>
            Transactions: {stats.total_transactions}
          </span>
        </div>
        <div style={{ 
          display: 'flex', 
          gap: isMobile ? '12px' : '17px', 
          alignItems: 'center',
          flexDirection: isMobile ? 'column' : 'row'
        }}>
          <span style={{ 
            fontSize: isMobile ? '15px' : '21px', 
            color: 'rgb(252, 217, 157)', 
            opacity: '0.8',
            fontWeight: 'bold'
          }}>
            Session: {sessionId ? sessionId.slice(0, 8) + '...' : 'None'}
          </span>
          <span style={{
            fontSize: isMobile ? '12px' : '14px',
            color: 'rgb(252, 217, 157)',
            backgroundColor: getConnectionStatusColor(),
            padding: '4px 8px',
            borderRadius: '3px',
            textTransform: 'uppercase',
            fontWeight: 'bold'
          }}>
            {connectionStatus}
          </span>
        </div>
      </div>

      <div style={{
        backgroundColor: 'rgb(252, 217, 157)',
        borderRadius: '5px',
        boxShadow: '0 4px 7px rgba(40, 40, 40, 1)',
        marginBottom: isMobile ? '24px' : '34px',
        padding: isMobile ? '12px' : '17px',
        animation: 'bubbleAppear 0.5s ease-in-out'
      }}>
        {isMobile ? (
          <div style={{ position: 'relative' }}>
            <button
              onClick={() => setDropdownOpen(!dropdownOpen)}
              style={{
                width: '100%',
                padding: '12px 17px',
                backgroundColor: 'rgb(169, 56, 56)',
                color: 'rgb(252, 217, 157)',
                border: 'none',
                borderRadius: '5px',
                fontWeight: 'bold',
                textTransform: 'uppercase',
                cursor: 'pointer',
                fontFamily: 'monospace',
                boxShadow: '0 4px 7px rgba(40, 40, 40, 1)',
                fontSize: '18px',
                display: 'flex',
                alignItems: 'center',
                justifyContent: 'space-between',
                minHeight: '44px'
              }}
            >
              <span>{tabs.find(tab => tab.id === activeTab)?.name || 'Select Tab'}</span>
              <ChevronDown 
                size={20} 
                style={{ 
                  transform: dropdownOpen ? 'rotate(180deg)' : 'rotate(0deg)',
                  transition: 'transform 0.3s ease'
                }} 
              />
            </button>
            
            {dropdownOpen && (
              <div style={{
                position: 'absolute',
                top: '100%',
                left: '0',
                right: '0',
                backgroundColor: 'rgb(252, 217, 157)',
                borderRadius: '5px',
                boxShadow: '0 4px 7px rgba(40, 40, 40, 1)',
                marginTop: '8px',
                zIndex: 10,
                overflow: 'hidden'
              }}>
                {tabs.map((tab, index) => (
                  <button
                    key={tab.id}
                    onClick={() => handleTabChange(tab.id)}
                    style={{
                      width: '100%',
                      padding: '12px 17px',
                      backgroundColor: activeTab === tab.id ? 'rgb(169, 56, 56)' : '#297cb3',
                      color: 'rgb(252, 217, 157)',
                      border: 'none',
                      borderBottom: index < tabs.length - 1 ? '1px solid rgba(40, 40, 40, 0.2)' : 'none',
                      fontWeight: 'bold',
                      textTransform: 'uppercase',
                      cursor: 'pointer',
                      fontFamily: 'monospace',
                      fontSize: '18px',
                      textAlign: 'left',
                      transition: 'background-color 0.3s ease'
                    }}
                    onMouseEnter={(e) => {
                      if (activeTab !== tab.id) {
                        e.target.style.backgroundColor = 'rgba(169, 56, 56, 0.8)';
                      }
                    }}
                    onMouseLeave={(e) => {
                      if (activeTab !== tab.id) {
                        e.target.style.backgroundColor = '#297cb3';
                      }
                    }}
                  >
                    {tab.name}
                  </button>
                ))}
              </div>
            )}
          </div>
        ) : (
          <nav style={{ 
            display: 'flex', 
            gap: '26px', 
            justifyContent: 'center',
            flexWrap: 'wrap'
          }}>
            {tabs.map(tab => (
              <button
                key={tab.id}
                onClick={() => setActiveTab(tab.id)}
                style={{
                  padding: '17px 34px',
                  border: activeTab === tab.id ? '3px solid rgb(169, 56, 56)' : '3px solid transparent',
                  borderRadius: '5px',
                  fontWeight: 'bold',
                  textTransform: 'uppercase',
                  backgroundColor: activeTab === tab.id ? 'rgb(169, 56, 56)' : '#297cb3',
                  color: 'rgb(252, 217, 157)',
                  cursor: 'pointer',
                  transition: 'transform 0.3s ease',
                  fontFamily: 'monospace',
                  boxShadow: '0 4px 7px rgba(40, 40, 40, 1)',
                  fontSize: '20px',
                  minHeight: '44px'
                }}
                onMouseEnter={(e) => e.target.style.transform = 'scale(1.1)'}
                onMouseLeave={(e) => e.target.style.transform = 'scale(1)'}
              >
                {tab.name}
              </button>
            ))}
          </nav>
        )}
      </div>

      <div style={{ padding: isMobile ? '0' : '0 5%' }}>
        {activeTab === 'marketplace' && (
          <div style={{ display: 'flex', flexDirection: 'column', gap: isMobile ? '24px' : '34px' }}>
            <div style={{
              backgroundColor: 'rgb(252, 217, 157)',
              borderRadius: '10px',
              boxShadow: '0 4px 7px rgba(40, 40, 40, 1)',
              animation: 'bubbleAppear 0.5s ease-in-out'
            }}>
              <div style={{
                padding: isMobile ? '17px' : '26px',
                borderBottom: '3px solid rgb(169, 56, 56)',
                textAlign: 'center'
              }}>
                <h3 style={{
                  color: 'rgb(169, 56, 56)',
                  fontWeight: 'bolder',
                  textTransform: 'uppercase',
                  margin: '0',
                  fontSize: isMobile ? '28px' : '39px'
                }}>
                  Slot Marketplace
                </h3>
              </div>
              <div style={{ padding: isMobile ? '17px' : '26px' }}>
                <div style={{
                  display: 'grid',
                  gridTemplateColumns: isMobile ? 'repeat(5, 1fr)' : 'repeat(10, 1fr)',
                  gap: isMobile ? '6px' : '8px',
                  marginBottom: isMobile ? '17px' : '26px'
                }}>
                  {slots.map(slot => (
                    <div
                      key={slot.slot_number}
                      style={{
                        ...getSlotStyle(slot.state),
                        padding: isMobile ? '8px' : '12px',
                        borderRadius: '5px',
                        textAlign: 'center',
                        cursor: 'pointer',
                        transition: 'transform 0.3s ease',
                        color: 'rgb(252, 217, 157)',
                        fontWeight: 'bold',
                        fontSize: isMobile ? '10px' : '12px',
                        boxShadow: '0 2px 4px rgba(40, 40, 40, 0.7)'
                      }}
                      onClick={() => {
                        if ((slot.slot_number >= currentSlot + 1 && slot.state === 'Available') || (typeof slot.state === 'object' && slot.state.AoTAuction)) {
                          submitAotBid(slot.slot_number);
                        }
                      }}
                      onMouseEnter={(e) => e.target.style.transform = 'scale(1.1)'}
                      onMouseLeave={(e) => e.target.style.transform = 'scale(1)'}
                    >
                      <div style={{ fontWeight: 'bolder', marginBottom: '2px' }}>{slot.slot_number}</div>
                      <div style={{ fontSize: isMobile ? '8px' : '10px' }}>{getStateName(slot.state)}</div>
                    </div>
                  ))}
                </div>
                
                <div style={{
                  display: 'flex',
                  gap: isMobile ? '12px' : '17px',
                  flexDirection: isMobile ? 'column' : 'row',
                  alignItems: isMobile ? 'stretch' : 'flex-start'
                }}>
                  <button
                    onClick={submitJitBid}
                    style={{
                      backgroundColor: '#297cb3',
                      color: 'rgb(252, 217, 157)',
                      padding: isMobile ? '17px' : '20px 34px',
                      border: 'none',
                      borderRadius: '5px',
                      fontWeight: 'bold',
                      textTransform: 'uppercase',
                      cursor: 'pointer',
                      fontFamily: 'monospace',
                      boxShadow: '0 4px 7px rgba(40, 40, 40, 1)',
                      transition: 'transform 0.3s ease',
                      fontSize: isMobile ? '18px' : '20px',
                      minHeight: '44px'
                    }}
                    onMouseEnter={(e) => e.target.style.transform = 'scale(1.05)'}
                    onMouseLeave={(e) => e.target.style.transform = 'scale(1)'}
                  >
                    Submit JIT Bid (Next Slot)
                  </button>
                  
                  <div style={{
                    backgroundColor: '#297cb3',
                    padding: isMobile ? '17px' : '20px',
                    borderRadius: '5px',
                    boxShadow: '0 4px 7px rgba(40, 40, 40, 1)',
                    flex: '1'
                  }}>
                    <h4 style={{
                      color: 'rgb(252, 217, 157)',
                      fontWeight: 'bold',
                      textTransform: 'uppercase',
                      margin: '0 0 12px 0',
                      fontSize: isMobile ? '16px' : '18px'
                    }}>
                      Legend
                    </h4>
                    <div style={{
                      display: 'grid',
                      gridTemplateColumns: isMobile ? '1fr 1fr' : 'repeat(3, 1fr)',
                      gap: isMobile ? '8px' : '12px',
                      fontSize: isMobile ? '14px' : '16px'
                    }}>
                      {[
                        { color: '#75bd4f', label: 'Available' },
                        { color: '#297cb3', label: 'JIT Auction' },
                        { color: 'rgb(169, 56, 56)', label: 'AOT Auction' },
                        { color: '#d97706', label: 'Reserved' },
                        { color: '#dc2626', label: 'Filled' },
                        { color: '#666', label: 'Expired' }
                      ].map(({ color, label }) => (
                        <div key={label} style={{ display: 'flex', alignItems: 'center', gap: '8px' }}>
                          <div style={{
                            width: '16px',
                            height: '16px',
                            backgroundColor: color,
                            borderRadius: '3px',
                            border: '1px solid rgba(40, 40, 40, 0.3)'
                          }}></div>
                          <span style={{
                            color: 'rgb(252, 217, 157)',
                            fontWeight: 'bold',
                            textTransform: 'uppercase'
                          }}>
                            {label}
                          </span>
                        </div>
                      ))}
                    </div>
                  </div>
                </div>
              </div>
            </div>
          </div>
        )}

        {activeTab === 'auctions' && (
          <div style={{ display: 'flex', flexDirection: 'column', gap: isMobile ? '24px' : '34px' }}>
            <div style={{ textAlign: 'center', animation: 'bubbleAppear 1s ease-in-out' }}>
              <h2 style={{
                color: 'rgb(169, 56, 56)',
                fontWeight: 'bolder',
                textTransform: 'uppercase',
                fontSize: isMobile ? '35px' : '51px',
                margin: '0 0 10px 0'
              }}>
                Active Auctions
              </h2>
              <p style={{
                color: 'rgb(169, 56, 56)',
                fontWeight: 'bold',
                fontSize: isMobile ? '20px' : '30px',
                margin: '0'
              }}>
                Bid for guaranteed slot inclusion
              </p>
            </div>

            <div style={{
              backgroundColor: 'rgb(252, 217, 157)',
              borderRadius: '10px',
              boxShadow: '0 4px 7px rgba(40, 40, 40, 1)',
              animation: 'bubbleAppear 0.5s ease-in-out'
            }}>
              <div style={{
                padding: isMobile ? '17px' : '26px',
                borderBottom: '3px solid rgb(169, 56, 56)',
                textAlign: 'center'
              }}>
                <h3 style={{
                  color: '#297cb3',
                  fontWeight: 'bolder',
                  textTransform: 'uppercase',
                  margin: '0',
                  fontSize: isMobile ? '24px' : '32px'
                }}>
                  JIT Auctions (Sealed-Bid)
                </h3>
              </div>
              <div style={{ padding: isMobile ? '17px' : '26px' }}>
                {jitAuctions.length === 0 ? (
                  <div style={{
                    textAlign: 'center',
                    color: 'rgb(169, 56, 56)',
                    fontWeight: 'bold',
                    textTransform: 'uppercase',
                    padding: isMobile ? '20px' : '30px',
                    fontSize: isMobile ? '18px' : '20px'
                  }}>
                    No active JIT auctions
                  </div>
                ) : (
                  jitAuctions.map(auction => (
                    <div key={auction.slot_number} style={{
                      backgroundColor: '#297cb3',
                      borderRadius: '5px',
                      boxShadow: '0 4px 7px rgba(40, 40, 40, 1)',
                      padding: isMobile ? '17px' : '20px',
                      marginBottom: isMobile ? '12px' : '17px'
                    }}>
                      <div style={{
                        display: 'flex',
                        justifyContent: 'space-between',
                        alignItems: 'center',
                        flexWrap: 'wrap',
                        gap: '12px'
                      }}>
                        <div>
                          <span style={{
                            color: 'rgb(252, 217, 157)',
                            fontWeight: 'bold',
                            textTransform: 'uppercase',
                            fontSize: isMobile ? '18px' : '22px'
                          }}>
                            Slot {auction.slot_number}
                          </span>
                          <div style={{
                            color: 'rgb(252, 217, 157)',
                            fontSize: isMobile ? '14px' : '16px',
                            opacity: '0.8',
                            marginTop: '4px'
                          }}>
                            Min Bid: {auction.min_bid} SOL
                          </div>
                        </div>
                        {auction.current_winner && (
                          <div style={{
                            color: '#75bd4f',
                            fontWeight: 'bold',
                            fontSize: isMobile ? '14px' : '16px'
                          }}>
                            Winner: {auction.current_winner[0].slice(0, 8)}... ({auction.current_winner[1]} SOL)
                          </div>
                        )}
                      </div>
                    </div>
                  ))
                )}
              </div>
            </div>

            <div style={{
              backgroundColor: 'rgb(252, 217, 157)',
              borderRadius: '10px',
              boxShadow: '0 4px 7px rgba(40, 40, 40, 1)',
              animation: 'bubbleAppear 0.6s ease-in-out'
            }}>
              <div style={{
                padding: isMobile ? '17px' : '26px',
                borderBottom: '3px solid rgb(169, 56, 56)',
                textAlign: 'center'
              }}>
                <h3 style={{
                  color: 'rgb(169, 56, 56)',
                  fontWeight: 'bolder',
                  textTransform: 'uppercase',
                  margin: '0',
                  fontSize: isMobile ? '24px' : '32px'
                }}>
                  AOT Auctions (English-Style)
                </h3>
              </div>
              <div style={{ padding: isMobile ? '17px' : '26px' }}>
                {aotAuctions.length === 0 ? (
                  <div style={{
                    textAlign: 'center',
                    color: 'rgb(169, 56, 56)',
                    fontWeight: 'bold',
                    textTransform: 'uppercase',
                    padding: isMobile ? '20px' : '30px',
                    fontSize: isMobile ? '18px' : '20px'
                  }}>
                    No active AOT auctions
                  </div>
                ) : (
                  aotAuctions.map(auction => (
                    <div key={auction.slot_number} style={{
                      backgroundColor: 'rgb(169, 56, 56)',
                      borderRadius: '5px',
                      boxShadow: '0 4px 7px rgba(40, 40, 40, 1)',
                      padding: isMobile ? '17px' : '20px',
                      marginBottom: isMobile ? '12px' : '17px'
                    }}>
                      <div style={{
                        display: 'flex',
                        justifyContent: 'space-between',
                        alignItems: 'center',
                        flexWrap: 'wrap',
                        gap: '12px'
                      }}>
                        <div>
                          <span style={{
                            color: 'rgb(252, 217, 157)',
                            fontWeight: 'bold',
                            textTransform: 'uppercase',
                            fontSize: isMobile ? '18px' : '22px'
                          }}>
                            Slot {auction.slot_number}
                          </span>
                          <div style={{
                            color: 'rgb(252, 217, 157)',
                            fontSize: isMobile ? '14px' : '16px',
                            opacity: '0.8',
                            marginTop: '4px'
                          }}>
                            Min: {auction.min_bid} SOL | Bids: {auction.bids_count}
                          </div>
                        </div>
                        <div>
                          {auction.highest_bid && (
                            <div style={{
                              color: '#75bd4f',
                              fontWeight: 'bold',
                              fontSize: isMobile ? '14px' : '16px',
                              marginBottom: '4px'
                            }}>
                              Highest: {auction.highest_bid} SOL
                            </div>
                          )}
                          <div style={{
                            color: 'rgb(252, 217, 157)',
                            fontSize: isMobile ? '12px' : '14px',
                            opacity: '0.7'
                          }}>
                            {auction.has_ended ? 'Ended' : 'Active'}
                          </div>
                        </div>
                      </div>
                    </div>
                  ))
                )}
              </div>
            </div>
          </div>
        )}

        {activeTab === 'transactions' && (
              <div style={{ display: 'flex', flexDirection: 'column', gap: isMobile ? '24px' : '34px' }}>
                <div style={{ textAlign: 'center', animation: 'bubbleAppear 1s ease-in-out' }}>
                  <h2 style={{
                    color: 'rgb(169, 56, 56)',
                    fontWeight: 'bolder',
                    textTransform: 'uppercase',
                    fontSize: isMobile ? '35px' : '51px',
                    margin: '0 0 10px 0'
                  }}>
                    Transaction History
                  </h2>
                  <p style={{
                    color: 'rgb(169, 56, 56)',
                    fontWeight: 'bold',
                    fontSize: isMobile ? '20px' : '30px',
                    margin: '0'
                  }}>
                    Track auction bids and inclusion status
                  </p>
                </div>

                <div style={{
                  backgroundColor: 'rgb(252, 217, 157)',
                  borderRadius: '10px',
                  boxShadow: '0 4px 7px rgba(40, 40, 40, 1)',
                  animation: 'bubbleAppear 0.5s ease-in-out'
                }}>
                  <div style={{
                    padding: isMobile ? '17px' : '26px',
                    borderBottom: '3px solid rgb(169, 56, 56)',
                    display: 'flex',
                    justifyContent: 'space-between',
                    alignItems: 'center',
                    flexWrap: 'wrap',
                    gap: '12px'
                  }}>
                    <h3 style={{
                      color: 'rgb(169, 56, 56)',
                      fontWeight: 'bolder',
                      textTransform: 'uppercase',
                      margin: '0',
                      fontSize: isMobile ? '24px' : '32px'
                    }}>
                      {showAllTransactions ? 'All Transactions' : 'My Transactions'}
                    </h3>
                    
                    <button
                      onClick={toggleTransactionView}
                      style={{
                        backgroundColor: showAllTransactions ? '#75bd4f' : '#297cb3',
                        color: 'rgb(252, 217, 157)',
                        padding: isMobile ? '8px 12px' : '10px 16px',
                        border: 'none',
                        borderRadius: '5px',
                        fontWeight: 'bold',
                        textTransform: 'uppercase',
                        cursor: 'pointer',
                        fontFamily: 'monospace',
                        boxShadow: '0 2px 4px rgba(40, 40, 40, 0.7)',
                        fontSize: isMobile ? '12px' : '14px',
                        display: 'flex',
                        alignItems: 'center',
                        gap: '6px',
                        transition: 'transform 0.3s ease'
                      }}
                      onMouseEnter={(e) => e.target.style.transform = 'scale(1.05)'}
                      onMouseLeave={(e) => e.target.style.transform = 'scale(1)'}
                    >
                      {showAllTransactions ? <Users size={16} /> : <User size={16} />}
                      {showAllTransactions ? 'Show Mine' : 'Show All'}
                    </button>
                  </div>
                  
                  <div style={{ padding: isMobile ? '17px' : '26px' }}>
                    {transactions.length === 0 ? (
                      <div style={{
                        textAlign: 'center',
                        color: 'rgb(169, 56, 56)',
                        fontWeight: 'bold',
                        textTransform: 'uppercase',
                        padding: isMobile ? '24px' : '34px',
                        fontSize: isMobile ? '20px' : '23px'
                      }}>
                        {showAllTransactions ? 'No transactions in the system yet!' : 'No transactions yet. Submit some bids!'}
                      </div>
                    ) : (
                      <>
                        {transactions.map(tx => (
                          <div key={tx.id} style={{
                            backgroundColor: '#297cb3',
                            borderRadius: '5px',
                            boxShadow: '0 4px 7px rgba(40, 40, 40, 1)',
                            padding: isMobile ? '17px' : '20px',
                            marginBottom: isMobile ? '12px' : '17px'
                          }}>
                            <div style={{
                              display: 'flex',
                              justifyContent: 'space-between',
                              alignItems: 'center',
                              marginBottom: '12px',
                              flexWrap: 'wrap',
                              gap: '8px'
                            }}>
                              <div style={{ display: 'flex', alignItems: 'center', gap: '8px' }}>
                                <span style={{
                                  color: '#75bd4f',
                                  fontWeight: 'bold',
                                  fontSize: isMobile ? '16px' : '18px'
                                }}>
                                  {tx.id.slice(0, 8)}...
                                </span>
                                {showAllTransactions && (
                                  <span style={{
                                    fontSize: isMobile ? '12px' : '14px',
                                    color: 'rgb(252, 217, 157)',
                                    backgroundColor: tx.sender === sessionId ? '#d97706' : 'rgba(252, 217, 157, 0.2)',
                                    padding: '2px 6px',
                                    borderRadius: '3px',
                                    fontWeight: 'bold'
                                  }}>
                                    {tx.sender === sessionId ? 'YOU' : tx.sender.slice(0, 6) + '...'}
                                  </span>
                                )}
                              </div>
                              <span style={{
                                padding: '4px 12px',
                                borderRadius: '3px',
                                fontSize: isMobile ? '12px' : '14px',
                                fontWeight: 'bold',
                                textTransform: 'uppercase',
                                backgroundColor: 
                                  getTransactionStatus(tx.status) === 'Included' ? '#75bd4f' :
                                  getTransactionStatus(tx.status) === 'Auction Won' ? '#d97706' :
                                  getTransactionStatus(tx.status) === 'Failed' ? '#dc2626' : '#666',
                                color: 'rgb(252, 217, 157)'
                              }}>
                                {getTransactionStatus(tx.status)}
                              </span>
                            </div>
                            <div style={{
                              display: 'grid',
                              gridTemplateColumns: isMobile ? '1fr' : '1fr 1fr',
                              gap: '8px',
                              fontSize: isMobile ? '14px' : '16px',
                              color: 'rgb(252, 217, 157)'
                            }}>
                              <div>
                                <strong>Type:</strong> {getTransactionType(tx.inclusion_type)}
                              </div>
                              <div>
                                <strong>Fee:</strong> {tx.priority_fee} SOL
                              </div>
                              <div>
                                <strong>Compute Units:</strong> {tx.compute_units}
                              </div>
                              <div>
                                <strong>Created:</strong> {new Date(tx.created_at).toLocaleTimeString()}
                              </div>
                            </div>
                          </div>
                        ))}
                        
                        {/* Pagination Controls */}
                        {pagination.total_pages > 1 && (
                          <div style={{
                            display: 'flex',
                            justifyContent: 'center',
                            alignItems: 'center',
                            gap: '12px',
                            marginTop: '20px',
                            flexWrap: 'wrap'
                          }}>
                            <button
                              onClick={() => handlePageChange(currentPage - 1)}
                              disabled={!pagination.has_prev}
                              style={{
                                backgroundColor: pagination.has_prev ? '#297cb3' : '#666',
                                color: 'rgb(252, 217, 157)',
                                border: 'none',
                                borderRadius: '5px',
                                padding: '8px 12px',
                                cursor: pagination.has_prev ? 'pointer' : 'not-allowed',
                                fontFamily: 'monospace',
                                fontWeight: 'bold',
                                display: 'flex',
                                alignItems: 'center',
                                gap: '4px'
                              }}
                            >
                              <ChevronLeft size={16} />
                              Prev
                            </button>
                            
                            <div style={{
                              color: 'rgb(169, 56, 56)',
                              fontWeight: 'bold',
                              fontSize: isMobile ? '14px' : '16px',
                              padding: '0 12px'
                            }}>
                              {currentPage} of {pagination.total_pages}
                            </div>
                            
                            <button
                              onClick={() => handlePageChange(currentPage + 1)}
                              disabled={!pagination.has_next}
                              style={{
                                backgroundColor: pagination.has_next ? '#297cb3' : '#666',
                                color: 'rgb(252, 217, 157)',
                                border: 'none',
                                borderRadius: '5px',
                                padding: '8px 12px',
                                cursor: pagination.has_next ? 'pointer' : 'not-allowed',
                                fontFamily: 'monospace',
                                fontWeight: 'bold',
                                display: 'flex',
                                alignItems: 'center',
                                gap: '4px'
                              }}
                            >
                              Next
                              <ChevronRight size={16} />
                            </button>
                          </div>
                        )}
                        
                        <div style={{
                          textAlign: 'center',
                          marginTop: '12px',
                          color: 'rgb(169, 56, 56)',
                          fontSize: isMobile ? '12px' : '14px',
                          fontWeight: 'bold'
                        }}>
                          Showing {transactions.length} of {pagination.total_count} transactions
                        </div>
                      </>
                    )}
                  </div>
                </div>
              </div>
            )}
          </div>

      <style>{`
        @keyframes fadeIn {
          from {
            opacity: 0;
          }
          to {
            opacity: 1;
          }
        }

        @keyframes bubbleAppear {
          from {
            transform: scale(0.1);
            opacity: 0;
          }
          to {
            transform: scale(1);
            opacity: 1;
          }
        }

        @keyframes spin {
          from { transform: rotate(0deg); }
          to { transform: rotate(360deg); }
        }
      `}</style>
    </div>
  );
};

export default RaikuSimulator;