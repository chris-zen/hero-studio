import * as React from 'react'
import {
  Navbar,
  Button,
  ButtonGroup,
  NumericInput,
  ControlGroup,
  InputGroup,
  // FormGroup
} from '@blueprintjs/core'

import MetronomeIcon from './svg/metronome.svg'

import './ToolBar.css'

export interface ToolBarProps {
}

export default class ToolBar extends React.Component<ToolBarProps, {}> {
  render() {
    return (
      <Navbar>
        <Navbar.Group>
          <ButtonGroup>
            <Button icon="play" />
            <Button icon="stop" />
            <Button icon="record" />
          </ButtonGroup>
        </Navbar.Group>
        <Navbar.Group
          style={{marginLeft: "12px"}}>
          <ControlGroup>
            <InputGroup
              style={{width: "100px", textAlign: "center"}}
              defaultValue="0000.00.00" />
          </ControlGroup>
        </Navbar.Group>
        <Navbar.Group
          style={{marginLeft: "12px"}}>
          <ControlGroup>
            <NumericInput
              style={{width: "45px", textAlign: "center"}}
              buttonPosition="none"
              value={120} max={480} majorStepSize={10} />
            <InputGroup
              style={{marginLeft: "6px", width: "72px", textAlign: "center"}}
              defaultValue="16 / 16" />
          </ControlGroup>
        </Navbar.Group>
        <Navbar.Group
          style={{marginLeft: "12px"}}>
          <ControlGroup>
            <Button style={{backgroundColor: "#A66321"}}>
              <MetronomeIcon />
            </Button>
            {/* <Button rightIcon="caret-down"></Button> */}
          </ControlGroup>
        </Navbar.Group>
        <Navbar.Group
          style={{marginLeft: "12px"}}>
          <ControlGroup>
            <Button icon="repeat" style={{backgroundColor: "#A66321"}}>
              {/* <MetronomeIcon /> */}
            </Button>
            <InputGroup
              style={{width: "100px", textAlign: "center"}}
              defaultValue="0000.00.00" />
            <InputGroup
              style={{width: "100px", textAlign: "center"}}
              defaultValue="0000.00.00" />
          </ControlGroup>
        </Navbar.Group>
      </Navbar>
    )
  }
}
