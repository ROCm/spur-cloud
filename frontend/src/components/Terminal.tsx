import { useEffect, useRef } from 'react';
import { Terminal as XTerm } from '@xterm/xterm';
import { FitAddon } from '@xterm/addon-fit';
import { WebLinksAddon } from '@xterm/addon-web-links';
import '@xterm/xterm/css/xterm.css';

interface Props {
  wsUrl: string;
}

export default function Terminal({ wsUrl }: Props) {
  const termRef = useRef<HTMLDivElement>(null);
  const xtermRef = useRef<XTerm | null>(null);
  const wsRef = useRef<WebSocket | null>(null);

  useEffect(() => {
    if (!termRef.current) return;

    const term = new XTerm({
      theme: {
        background: '#0d1117',
        foreground: '#c9d1d9',
        cursor: '#58a6ff',
        selectionBackground: '#264f78',
      },
      fontFamily: '"JetBrains Mono", "Fira Code", monospace',
      fontSize: 14,
      cursorBlink: true,
    });
    xtermRef.current = term;

    const fitAddon = new FitAddon();
    term.loadAddon(fitAddon);
    term.loadAddon(new WebLinksAddon());

    term.open(termRef.current);
    fitAddon.fit();

    // Connect WebSocket
    const ws = new WebSocket(wsUrl);
    wsRef.current = ws;

    ws.onopen = () => {
      term.writeln('\x1b[32mConnected to session.\x1b[0m\r\n');
    };

    ws.onmessage = (event) => {
      term.write(event.data);
    };

    ws.onclose = () => {
      term.writeln('\r\n\x1b[31mConnection closed.\x1b[0m');
    };

    ws.onerror = () => {
      term.writeln('\r\n\x1b[31mConnection error.\x1b[0m');
    };

    // Send keystrokes to WebSocket
    term.onData((data) => {
      if (ws.readyState === WebSocket.OPEN) {
        ws.send(data);
      }
    });

    // Handle resize
    const resizeObserver = new ResizeObserver(() => {
      fitAddon.fit();
    });
    resizeObserver.observe(termRef.current);

    return () => {
      resizeObserver.disconnect();
      ws.close();
      term.dispose();
    };
  }, [wsUrl]);

  return (
    <div
      ref={termRef}
      className="w-full h-[600px] bg-[#0d1117] rounded-lg overflow-hidden border border-gray-800"
    />
  );
}
