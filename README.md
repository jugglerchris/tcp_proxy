Simple TCP Proxy

The motivating use case is a hosting provider which requires you to run a
backend server (such as a Python site using gunicorn) behind their HTTPS proxy;
but requires that your server listens on an external IP address/port.

This doesn't seem ideal (e.g. harder to trust the proxy headers), and gunicorn
doesn't support an allow list; so this proxy simply listens on an address/port
but drops any connection not from a list of allowed peers.
