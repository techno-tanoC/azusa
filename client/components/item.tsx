import style from './item.module.css'

const deleteItem = id => {
  fetch(
    `http://localhost:8000/downloads/${id}`,
    {
      method: "DELETE"
    }
  )
}

export default function Item({ id, name, total, size }) {
  const percent = Math.floor(size * 100 / total)

  return (
    <div>
      <span style={{ width: `${percent}%` }} />
      <div>
        <div>
          {name}
        </div>
        <div>
          <button onClick={() => deleteItem(id)}>
          </button>
        </div>
        <div>
          {percent}%
          <br />
          {size.toLocaleString()} / {total.toLocaleString()}
        </div>
      </div>
    </div>
  )
}
