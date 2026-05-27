import React from 'react';
import ReactDOM from 'react-dom/client';
import { App } from './App';
import { DictationOverlay } from './DictationOverlay';
import './styles.css';

const params = new URLSearchParams(window.location.search);
const isOverlayView = params.get('view') === 'overlay';
document.documentElement.classList.toggle('overlay-document', isOverlayView);
document.body.classList.toggle('overlay-document', isOverlayView);
const Root = isOverlayView ? DictationOverlay : App;

ReactDOM.createRoot(document.getElementById('root') as HTMLElement).render(
  <React.StrictMode>
    <Root />
  </React.StrictMode>,
);
