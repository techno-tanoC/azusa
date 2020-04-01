import React from 'react'
import Item from '../Item'

export default class ItemList extends React.Component {
  constructor(props) {
    super(props)
    this.state = {data: []}
    this.load = this.load.bind(this)
  }

  load() {
    console.log(this.state)
    fetch("http://localhost:3000/download", {
      mode: 'cors'
    })
      .then(res => res.json())
      .then(json => this.setState({data: json}))
  }

  componentDidMount() {
    this.load()
    setInterval(this.load, 1000)
  }

  render() {
    return (
      <div>
        {
          this.state.data.map(item => (
            <Item key={item.id} {...item} />
          ))
        }
      </div>
    )
  }
}
