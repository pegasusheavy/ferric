import React from 'react';
import ReactDOM from 'react-dom/client';
import App from './App';
import './styles.css';

// Performance measurement
window.__BENCHMARK__ = {
  startTime: performance.now(),
  marks: {},
  mark(name) {
    this.marks[name] = performance.now();
  },
  measure(name, start, end) {
    const startTime = this.marks[start] || 0;
    const endTime = this.marks[end] || performance.now();
    return endTime - startTime;
  },
};

window.__BENCHMARK__.mark('init');

ReactDOM.createRoot(document.getElementById('root')).render(
  <React.StrictMode>
    <App />
  </React.StrictMode>
);

window.__BENCHMARK__.mark('rendered');


import ReactDOM from 'react-dom/client';
import App from './App';
import './styles.css';

// Performance measurement
window.__BENCHMARK__ = {
  startTime: performance.now(),
  marks: {},
  mark(name) {
    this.marks[name] = performance.now();
  },
  measure(name, start, end) {
    const startTime = this.marks[start] || 0;
    const endTime = this.marks[end] || performance.now();
    return endTime - startTime;
  },
};

window.__BENCHMARK__.mark('init');

ReactDOM.createRoot(document.getElementById('root')).render(
  <React.StrictMode>
    <App />
  </React.StrictMode>
);

window.__BENCHMARK__.mark('rendered');

