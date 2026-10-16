import React from 'react';
import {createRoot} from 'react-dom/client';
import {App} from './App';
import {Picker} from './Picker';
import '@dimalabs/react-component-flat-product/themes.css';
import '@dimalabs/react-component-flat-product/styles.css';
import './styles.css';
createRoot(document.getElementById('root')!).render(<React.StrictMode>{new URLSearchParams(location.search).get('surface')==='picker'?<Picker/>:<App/>}</React.StrictMode>);
