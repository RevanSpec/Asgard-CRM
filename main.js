import { app, BrowserWindow, ipcMain } from 'electron';
import path from 'path';
import { fileURLToPath } from 'url';
import nodemailer from 'nodemailer';

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

  // Focus the window once it is ready to prevent clicks from being ignored
  win.once('ready-to-show', () => {
    win.show();
    win.focus();
  });

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

// IPC Handler to send emails using nodemailer
ipcMain.handle('send-email', async (event, { smtpConfig, emailData }) => {
  const { host, port, user, pass, secure, from } = smtpConfig;
  const { to, subject, text, filename, pdfBase64 } = emailData;

  // Create transporter
  const transporter = nodemailer.createTransport({
    host,
    port: parseInt(port),
    secure: secure === 'ssl', // true for port 465, false for others
    auth: {
      user,
      pass,
    },
    // Proton Mail Bridge uses a self-signed certificate locally,
    // so we must bypass certificate validation for localhost/127.0.0.1
    tls: {
      rejectUnauthorized: host !== '127.0.0.1' && host !== 'localhost',
    },
  });

  const attachments = [];
  if (pdfBase64 && filename) {
    attachments.push({
      filename,
      content: Buffer.from(pdfBase64, 'base64'),
    });
  }

  try {
    const info = await transporter.sendMail({
      from: from || user,
      to,
      subject,
      text,
      attachments,
    });
    return { success: true, messageId: info.messageId };
  } catch (error) {
    console.error('SMTP Send Error:', error);
    return { success: false, error: error.message };
  }
});

// IPC Handler to verify SMTP configuration
ipcMain.handle('test-smtp', async (event, smtpConfig) => {
  const { host, port, user, pass, secure } = smtpConfig;
  
  const transporter = nodemailer.createTransport({
    host,
    port: parseInt(port),
    secure: secure === 'ssl',
    auth: {
      user,
      pass,
    },
    tls: {
      rejectUnauthorized: host !== '127.0.0.1' && host !== 'localhost',
    },
  });

  try {
    await transporter.verify();
    return { success: true };
  } catch (error) {
    console.error('SMTP Verify Error:', error);
    return { success: false, error: error.message };
  }
});
