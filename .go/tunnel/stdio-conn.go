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

func (c *stuStdioConn) Read(b []byte) (n int, err error)   { return c.r.Read(b) }
func (c *stuStdioConn) Write(b []byte) (n int, err error)  { return c.w.Write(b) }
func (c *stuStdioConn) Close() error                       { return nil }
func (c *stuStdioConn) LocalAddr() net.Addr                { return nil }
func (c *stuStdioConn) RemoteAddr() net.Addr               { return nil }
func (c *stuStdioConn) SetDeadline(t time.Time) error      { return nil }
func (c *stuStdioConn) SetReadDeadline(t time.Time) error  { return nil }
func (c *stuStdioConn) SetWriteDeadline(t time.Time) error { return nil }
