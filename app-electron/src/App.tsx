import * as React from 'react'

import { FocusStyleManager } from "@blueprintjs/core";

import ToolBar from './ToolBar'
import Workspace from './Workspace'
import StatusBar from './StatusBar'

FocusStyleManager.onlyShowFocusOnTabs();

import './style.scss'
import './App.css'

export interface AppProps {
}

export class App extends React.Component<AppProps, {}> {
  render() {
    return (
      <div className='container bp3-dark'>
        <ToolBar />
        <Workspace />
        <StatusBar />
      </div>
    )
  }
}
