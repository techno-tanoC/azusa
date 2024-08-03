import { useState, useEffect } from 'react'
import { createRoot } from 'react-dom/client'

const fetchItem = async () => {
  const url = "/downloads"
  const res = await fetch(url)
  const json = await res.json()
  return json
}

const deleteItem = async (id) => {
  const url = `/downloads/${id}`
  await fetch(url, { method: "DELETE" })
}

const Item = ({id, name, total, size}) => {
  const percent = `${Math.floor(size * 100 / total)}%`
  const ratio = `${size.toLocaleString()} / ${total.toLocaleString()}`
  return (
    <div className="card">
      <div className="body">
        <span style={ { width: percent } } className="progress" />
        <div className="name">
          { name }
        </div>
        <div className="counts">
          { percent }
          <br />
          { ratio }
        </div>
      </div>
      <button className="button" onClick={ () => deleteItem(id) }>
        cancel
      </button>
    </div>
  )
}

const ItemList = ({items}) => {
  return (
    <div className="list">
      {
        items.map(item => (
          <Item key={ item.id } id={ item.id } name={ item.name } total={ item.total } size={ item.size } />
        ))
      }
    </div>
  )
}

const Page = () => {
  const [items, setItems] = useState([])

  useEffect(() => {
    const f = async () => {
      const is = await fetchItem()
      setItems(is)
    }

    f()
    const intervalId = setInterval(f, 1000)
    return () => clearInterval(intervalId)
  }, [])

  return (
    <ItemList items={ items } />
  )
}

const container = document.querySelector("#root")
const root = createRoot(container)
root.render(<Page />)
