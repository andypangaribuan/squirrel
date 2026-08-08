/*
 * Copyright (c) 2026.
 * Created by Andy Pangaribuan (iam.pangaribuan@gmail.com)
 * https://github.com/apangaribuan
 *
 * This product is protected by copyright and distributed under
 * licenses restricting copying, distribution and decompilation.
 * All Rights Reserved.
 */

package tunnel

import (
	"net"
	"time"
)

func (p *stuProxyConn) LocalAddr() net.Addr                { return &net.TCPAddr{} }
func (p *stuProxyConn) RemoteAddr() net.Addr               { return &net.TCPAddr{} }
func (p *stuProxyConn) SetDeadline(t time.Time) error      { return nil }
func (p *stuProxyConn) SetReadDeadline(t time.Time) error  { return nil }
func (p *stuProxyConn) SetWriteDeadline(t time.Time) error { return nil }
func (p *stuProxyConn) Close() error {
	_ = p.cmd.Process.Kill()
	return p.cmd.Wait()
}
