rust-scan
=========


.. contents:: 


主流扫描模式
---------------

*   TCP-SYN     ( 收到 SYN 包为开放， RST 包为关闭)
*   TCP-ACK     ( 收到 RST 包为开放，未过滤 )
*   TCP-Connect
*   ICMPEcho    ( 主机是否在线 )
*   UDP         ( ... )

其它工具介绍
---------------

*   `nmap/nmap <https://github.com/nmap/nmap>`_
*   `zmap/zmap <https://github.com/zmap/zmap>`_
*   `robertdavidgraham/masscan <https://github.com/robertdavidgraham/masscan>`_

