import React from 'react';
import {createRoot} from 'react-dom/client';
import {App} from './App';
import {Picker} from './Picker';
import './theme.css';
import './components/styles.css';
import './styles.css';
createRoot(document.getElementById('root')!).render(<React.StrictMode>{new URLSearchParams(location.search).get('surface')==='picker'?<Picker/>:<App/>}</React.StrictMode>);
