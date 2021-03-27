import style from './download.module.scss'

const deleteItem = (id: string) => {
  fetch(
    `http://localhost:8080/downloads/${id}`,
    {
      method: "DELETE"
    }
  )
}

export default function Download({
  id, name, total, size
}: {
  id: string, name: string, total: number, size: number
}) {
  const percent = `${Math.floor(size * 100 / total)}%`
  const ratio = `${size.toLocaleString()} / ${total.toLocaleString()}`

  return (
    <div className={style.card}>
      <div className={style.body}>
        <span style={{ width: percent }} className={style.progress} />
        <div className={style.name}>
          {name}
        </div>
        <div className={style.counts}>
          {percent}
          <br />
          {ratio}
        </div>
      </div>
      <button className={style.button} onClick={() => deleteItem(id)}>
        cancel
      </button>
    </div>
  )
}
