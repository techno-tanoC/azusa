import DownloadList from '../components/download_list'

import {useState, useEffect} from 'react'

const fetchDownloads = async () => {
  const res = await fetch("http://localhost:8080/downloads")
  const json = await res.json()
  return json.downloads
}

export default function Index() {
  const [downloads, setDownloads] = useState([])

  useEffect(() => {
    const f = async () => {
      const news = await fetchDownloads()
      setDownloads(news)
    }
    // setInterval(f, 1000)
    f()
  }, [setDownloads])

  return (
    <DownloadList downloads={downloads} />
  )
}
