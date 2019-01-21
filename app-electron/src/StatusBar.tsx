import * as React from 'react'

import './StatusBar.css'

export interface StatusBarProps {
}

export default class StatusBar extends React.Component<StatusBarProps, {}> {
  render() {
    return (
      <div className='statusbar'>
        Status bar
      </div>
    )
  }
}
