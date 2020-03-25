import React from 'react'
import style from './style.css'

const deleteItem = id => {
  fetch(`http://localhost:3000/download?id=${id}`, {
    method: 'DELETE'
  })
}

export default ({id, name, total, size}) => {
  const percent = Math.floor(size * 100 / total)

  return (
    <div className={style.card}>
      <span style={ {width: `${percent}%`} } className={style.progress}></span>
      <div className={style.content}>
        <div className={style.name}>
          {name}
        </div>
        <div className={style.row}>
          <div>
            <button className={style.button} onClick={() => deleteItem(id)}>cancel</button> {percent}%
          </div>
          <div>
            {size.toLocaleString()} / {total.toLocaleString()}
          </div>
        </div>
      </div>
    </div>
  )
}
