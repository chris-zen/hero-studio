import * as React from 'react'
// import SplitterLayout from 'react-splitter-layout'

// import { ReactComponent as MetronomeIcon } from './svg/metronome.svg'

import './Workspace.css'

export default class Workspace extends React.Component {
  render () {
    return (
      <div className='workspace'>
        {/* <MetronomeIcon /> */}
        {/* <SplitterLayout customClassName='vsplitter' primaryIndex={1} secondaryInitialSize={220}>
          <div>Left</div>
          <SplitterLayout customClassName='hsplitter' vertical percentage secondaryInitialSize={50}>
            <div>Top</div>
            <div>Bottom</div>
          </SplitterLayout>
        </SplitterLayout> */}
      </div>
    )
  }
}
