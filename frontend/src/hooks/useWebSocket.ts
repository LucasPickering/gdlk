import { useCallback, useEffect, useRef, useState } from 'react';
import useSafeCallbacks from './useSafeCallbacks';
import useIsMounted from './useIsMounted';

const RECONNECT_TIMEOUT = 5000;

export type SocketSend<T> = (data: T) => void;
export type SocketEventConsumer<E, Out> = (
  send: SocketSend<Out>,
  event: E
) => void;
export type SocketConnectionStatus =
  | 'connecting'
  | 'connected'
  | 'closedError'
  | 'closedNormal';

interface SocketCallbacks<In, Out> {
  onOpen?: SocketEventConsumer<Event, Out>;
  onMessage?: SocketEventConsumer<In, Out>;
  onError?: SocketEventConsumer<Event, Out>;
  onClose?: SocketEventConsumer<CloseEvent, Out>;
}

/**
 * Hook for managing a websocket. Connects to the given address, if it is not
 * empty. If the address changes, the socket will close and re-connect.
 *
 * ### Types
 *
 * `In` - The type of data received over the websocket. Received data will be
 *    coerced to this type, so if the API doesn't match, that's a bummer.
 * `Out` - The type of data sent out over the websocket.
 */
const useWebSocket = <In, Out>(
  address: string | undefined,
  callbacks: SocketCallbacks<In, Out>,
  dependencies: readonly unknown[] = []
): { status: SocketConnectionStatus; send: SocketSend<Out> } => {
  const [status, setStatus] = useState<SocketConnectionStatus>('connecting');
  const wsRef = useRef<WebSocket | undefined>();
  const reconnectTimeoutIdRef = useRef<number | undefined>();

  // Memoized send function
  const send = useCallback<SocketSend<Out>>((data: unknown) => {
    const { current: ws } = wsRef;
    if (ws) {
      ws.send(JSON.stringify(data));
    } else {
      throw new Error('send called while websocket is closed');
    }
  }, []);

  // Prevent updates after unmounting
  const isMounted = useIsMounted();

  const { onOpen, onMessage, onError, onClose } = useSafeCallbacks<
    SocketCallbacks<In, Out>
  >(callbacks);

  /**
   * A function to establish a WS connection
   */
  const connect = useCallback(
    (addr: string): (() => void) => {
      const protocol = window.location.protocol === 'http:' ? 'ws' : 'wss';
      const fullAddr = `${protocol}://${window.location.host}${addr}`;
      wsRef.current = new WebSocket(fullAddr);

      const { current: ws } = wsRef;
      wsRef.current.onopen = (event) => {
        if (isMounted.current) {
          setStatus('connected');
          if (onOpen) {
            onOpen(send, event);
          }
        }
      };
      ws.onmessage = (event) => {
        if (isMounted.current && onMessage) {
          onMessage(send, JSON.parse(event.data));
        }
      };
      ws.onerror = (event) => {
        console.log(event);
        if (isMounted.current && onError) {
          onError(send, event);
        }
      };

      ws.onclose = (event) => {
        if (isMounted.current) {
          // code === 1000 indicates normal closure
          if (event.code === 1000) {
            setStatus('closedNormal');
          } else {
            setStatus('closedError');
            // Reconnect after a certain amount of time
            reconnectTimeoutIdRef.current = window.setTimeout(() => {
              connect(addr);
            }, RECONNECT_TIMEOUT);
          }

          if (onClose) {
            onClose(send, event);
          }
        }
      };

      // Close the socket on unmount
      return () => {
        window.clearTimeout(reconnectTimeoutIdRef.current);
        ws.close();
      };
    },
    [onOpen, onMessage, onError, onClose, send, isMounted]
  );

  useEffect(() => {
    // Only connect if a real address is given.
    if (address) {
      return connect(address);
    }
  }, [
    connect,
    address,
    ...dependencies, // eslint-disable-line react-hooks/exhaustive-deps
  ]);

  return { status, send };
};

export default useWebSocket;
