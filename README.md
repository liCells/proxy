# proxy

简易的代理系统

```
cargo run [path]
```


```json5
{
  "proxy_group": [
    {
      // 线程池大小
      "thread_pool_size": 4,
      // 可访问IP
      "bind": "0.0.0.0",
      // 监听端口
      "port": 12004,
      // 代理上下文
      "context": "/",
      // 代理路径
      "path": "/Users/lz/code/projects/github/proxy/html/",
      // 默认跳转路径
      "index": "index.html",
      // TODO 超时时间 unit: s
      "timeout": 3,
      // TODO 是否启用缓存
      "cache": false,
      // 404 页面
      "404": "404.html"
    }
  ]
}
```
