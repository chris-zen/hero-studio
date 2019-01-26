import { app, BrowserWindow } from 'electron'

// require('electron-reload')(__dirname)

declare var __dirname: string

let mainWindow: Electron.BrowserWindow

function onReady() {
  mainWindow = new BrowserWindow({
    width: 800,
    height: 600,
    // titleBarStyle: 'hiddenInset',
    // title: 'Hero Studio',
    backgroundColor: '#333333',
    // Don't show the window until it's ready, this prevents any white flickering
    // show: false,
    webPreferences: {
      nodeIntegration: false,
      contextIsolation: true
    }
  })

  const fileName = `file://${__dirname}/index.html`
  console.log(`fileName=${fileName}`)

  mainWindow.loadURL(fileName)
  mainWindow.on('close', () => app.quit())
}

app.on('ready', () => onReady())
app.on('window-all-closed', () => app.quit())
console.log(`Electron Version ${app.getVersion()}`)
