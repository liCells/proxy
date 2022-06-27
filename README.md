# proxy

简易的代理系统

```
cargo run [path]
```


```json5
{
  "proxy_group": [
    {
      // 可访问IP
      "bind": "0.0.0.0",
      // 监听端口
      "port": 1204,
      // TODO 超时时间 unit: s
      "timeout": 3,
      // TODO 是否启用缓存
      "cache": false,
      // 线程池大小
      "thread_pool_size": 4,
      "rules": {
        // 代理上下文
        "/": {
          // 代理路径
          "path": "/proxy/html",
          // 默认跳转路径
          "index": "/index.html",
          // 404 页面
          "not_found_page": "/proxy/html/404.html",
          // 访问日志
          "access_log": "/proxy/logs/access.log"
        }
      }
    }
  ]
}
```
