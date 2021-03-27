import Item from './download'
import style from './download_list.module.scss'

export default function DownloadList({ downloads }) {
  return (
    <div className={style.list}>
      {
        downloads.map(download => (
          <Item key={download.id} {...download} />
        ))
      }
    </div>
  )
}
