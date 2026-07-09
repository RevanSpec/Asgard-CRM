import { app, BrowserWindow } from 'electron';
import path from 'path';
import { fileURLToPath } from 'url';

const __filename = fileURLToPath(import.meta.url);
const __dirname = path.dirname(__filename);

function createWindow() {
  const win = new BrowserWindow({
    width: 1300,
    height: 850,
    minWidth: 1000,
    minHeight: 700,
    webPreferences: {
      nodeIntegration: true,
      contextIsolation: false,
    },
    title: 'Asgard CRM',
  });

  // Remove the default menu bar
  win.setMenuBarVisibility(false);

  const isDev = !app.isPackaged;
  if (isDev) {
    const loadURLWithRetry = () => {
      win.loadURL('http://localhost:5173').catch(() => {
        setTimeout(loadURLWithRetry, 500);
      });
    };
    loadURLWithRetry();
    // Open DevTools in dev mode
    win.webContents.openDevTools();
  } else {
    win.loadFile(path.join(__dirname, 'dist/index.html'));
  }
}

app.whenReady().then(createWindow);

app.on('window-all-closed', () => {
  if (process.platform !== 'darwin') {
    app.quit();
  }
});

app.on('activate', () => {
  if (BrowserWindow.getAllWindows().length === 0) {
    createWindow();
  }
});
