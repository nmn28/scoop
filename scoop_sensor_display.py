import time
import serial
import serial.tools.list_ports
from http.server import HTTPServer, SimpleHTTPRequestHandler
import socketserver
import threading
import json
import websockets
import asyncio
import logging
import re
import socket
import os
import signal
import subprocess

# Configure logging
logging.basicConfig(level=logging.DEBUG)
logger = logging.getLogger(__name__)

# Global variable to store latest reading
latest_reading = {
    "red": 0,
    "ir": 0,
    "timestamp": time.time(),
    "status": "Disconnected"
}

def kill_process_on_port(port):
    """Kill any process using the specified port."""
    try:
        # Use lsof to find process using the port
        cmd = f"lsof -i :{port} -t"
        output = subprocess.check_output(cmd, shell=True)
        pid = int(output.strip())
        os.kill(pid, signal.SIGTERM)
        time.sleep(0.1)  # Give process time to terminate
        return True
    except (subprocess.CalledProcessError, ValueError):
        # No process found using the port
        return False
    except Exception as e:
        logger.error(f"Error killing process on port {port}: {e}")
        return False

def find_xiao_port():
    """Find the XIAO nRF52840 port."""
    ports = serial.tools.list_ports.comports()
    for port in ports:
        logger.info(f"Found port: {port.device} - {port.description}")
        if "XIAO nRF52840 Sense" in port.description:
            return port.device
    return None

async def websocket_handler(websocket, path):
    """Handle websocket connections."""
    try:
        while True:
            # Send the latest reading
            await websocket.send(json.dumps(latest_reading))
            await asyncio.sleep(0.1)  # Send updates every 100ms
    except websockets.exceptions.ConnectionClosed:
        pass

def run_websocket_server():
    """Run the websocket server."""
    # Kill any existing process using the WebSocket port
    if kill_process_on_port(8765):
        logger.info("Killed existing process using WebSocket port 8765")
    
    loop = asyncio.new_event_loop()
    asyncio.set_event_loop(loop)
    
    try:
        start_server = websockets.serve(
            websocket_handler, 
            "localhost", 
            8765,
            reuse_address=True
        )
        loop.run_until_complete(start_server)
        loop.run_forever()
    except OSError as e:
        if e.errno == 48:  # Address already in use
            logger.error("WebSocket port 8765 is still in use. Please manually stop any running instances.")
        else:
            logger.error(f"WebSocket server error: {e}")
        return

class HttpHandler(SimpleHTTPRequestHandler):
    def do_GET(self):
        if self.path == '/':
            self.path = '/index.html'
        return SimpleHTTPRequestHandler.do_GET(self)

    def log_message(self, format, *args):
        # Suppress logging of HTTP requests
        pass

class ReuseAddressHTTPServer(socketserver.TCPServer):
    allow_reuse_address = True

def run_http_server():
    """Run the HTTP server."""
    # Kill any existing process using the HTTP port
    if kill_process_on_port(8000):
        logger.info("Killed existing process using HTTP port 8000")
    
    try:
        with ReuseAddressHTTPServer(("", 8000), HttpHandler) as httpd:
            logger.info("HTTP server running on port 8000")
            httpd.serve_forever()
    except OSError as e:
        if e.errno == 48:  # Address already in use
            logger.error("HTTP port 8000 is still in use. Please manually stop any running instances.")
        else:
            logger.error(f"HTTP server error: {e}")
        return

def create_html_file():
    """Create the HTML frontend."""
    html_content = """
<!DOCTYPE html>
<html>
<head>
    <title>Heart Rate Monitor</title>
    <style>
        body {
            font-family: Arial, sans-serif;
            margin: 0;
            padding: 20px;
            background-color: #f0f0f0;
        }
        .container {
            max-width: 800px;
            margin: 0 auto;
            background-color: white;
            padding: 20px;
            border-radius: 10px;
            box-shadow: 0 2px 5px rgba(0,0,0,0.1);
        }
        .readings {
            display: flex;
            justify-content: space-around;
            margin: 20px 0;
        }
        .reading-box {
            text-align: center;
            padding: 15px;
            border-radius: 8px;
            background-color: #f8f8f8;
            min-width: 200px;
        }
        .reading-label {
            font-size: 16px;
            color: #666;
            margin-bottom: 5px;
        }
        .reading-value {
            font-size: 32px;
            color: #333;
        }
        .status {
            text-align: center;
            padding: 10px;
            margin-bottom: 20px;
            border-radius: 5px;
        }
        .status.connected {
            background-color: #d4edda;
            color: #155724;
        }
        .status.disconnected {
            background-color: #f8d7da;
            color: #721c24;
        }
        .timestamp {
            text-align: center;
            color: #666;
            margin-bottom: 20px;
        }
        .instructions {
            text-align: center;
            padding: 15px;
            margin: 20px 0;
            background-color: #e7f3fe;
            border-radius: 5px;
            color: #004085;
        }
        #chart {
            width: 100%;
            height: 300px;
            margin-top: 20px;
        }
        #debug {
            font-family: monospace;
            margin-top: 20px;
            padding: 10px;
            background-color: #f8f8f8;
            border-radius: 5px;
            height: 150px;
            overflow-y: auto;
        }
    </style>
    <script src="https://cdn.plot.ly/plotly-latest.min.js"></script>
</head>
<body>
    <div class="container">
        <h1 style="text-align: center;">Heart Rate Monitor</h1>
        <div id="status" class="status disconnected">Disconnected</div>
        <div class="instructions">
            <p><strong>Instructions:</strong></p>
            <p>1. Place your finger gently but firmly on the sensor</p>
            <p>2. Hold still for 5-10 seconds to allow readings to stabilize</p>
            <p>3. Maintain consistent pressure - not too hard, not too soft</p>
        </div>
        <div class="timestamp" id="timestamp">--</div>
        <div class="readings">
            <div class="reading-box">
                <div class="reading-label">RED</div>
                <div class="reading-value" id="red-value">--</div>
            </div>
            <div class="reading-box">
                <div class="reading-label">IR</div>
                <div class="reading-value" id="ir-value">--</div>
            </div>
        </div>
        <div id="chart"></div>
        <div id="debug"></div>
    </div>

    <script>
        let ws = new WebSocket('ws://localhost:8765');
        let redReadings = [];
        let irReadings = [];
        let timestamps = [];
        let debugDiv = document.getElementById('debug');

        // Initialize the chart
        let trace1 = {
            y: redReadings,
            x: timestamps,
            name: 'RED',
            mode: 'lines',
            line: {color: '#ff4444'}
        };

        let trace2 = {
            y: irReadings,
            x: timestamps,
            name: 'IR',
            mode: 'lines',
            line: {color: '#4444ff'}
        };

        let layout = {
            title: 'Sensor Readings Over Time',
            xaxis: {title: 'Time'},
            yaxis: {title: 'Value'}
        };

        Plotly.newPlot('chart', [trace1, trace2], layout);

        ws.onmessage = function(event) {
            let data = JSON.parse(event.data);
            
            // Update status
            let statusDiv = document.getElementById('status');
            statusDiv.textContent = data.status;
            statusDiv.className = 'status ' + (data.status === 'Connected' ? 'connected' : 'disconnected');

            // Update current readings
            document.getElementById('red-value').textContent = data.red;
            document.getElementById('ir-value').textContent = data.ir;
            document.getElementById('timestamp').textContent = 
                new Date(data.timestamp * 1000).toLocaleTimeString();

            // Update debug display
            let newLine = document.createElement('div');
            newLine.textContent = `${new Date(data.timestamp * 1000).toLocaleTimeString()}: RED=${data.red} IR=${data.ir}`;
            debugDiv.appendChild(newLine);
            if (debugDiv.childNodes.length > 50) {
                debugDiv.removeChild(debugDiv.firstChild);
            }
            debugDiv.scrollTop = debugDiv.scrollHeight;

            // Update chart
            redReadings.push(data.red);
            irReadings.push(data.ir);
            timestamps.push(new Date(data.timestamp * 1000));

            // Keep last 30 readings
            if (redReadings.length > 30) {
                redReadings.shift();
                irReadings.shift();
                timestamps.shift();
            }

            Plotly.update('chart', {
                y: [redReadings, irReadings],
                x: [timestamps, timestamps]
            });
        };

        ws.onclose = function() {
            document.getElementById('status').textContent = 'Disconnected';
            document.getElementById('status').className = 'status disconnected';
        };

        // Attempt to reconnect if connection is lost
        ws.onclose = function() {
            document.getElementById('status').textContent = 'Disconnected - Attempting to reconnect...';
            document.getElementById('status').className = 'status disconnected';
            setTimeout(function() {
                ws = new WebSocket('ws://localhost:8765');
            }, 1000);
        };
    </script>
</body>
</html>
    """
    with open('index.html', 'w') as f:
        f.write(html_content)

def read_serial_data(port):
    """Read data from serial port and update latest_reading."""
    global latest_reading
    
    try:
        with serial.Serial(port, 115200, timeout=1) as ser:
            logger.info(f"Connected to {port}")
            latest_reading["status"] = "Connected"
            
            while True:
                if ser.in_waiting:
                    line = ser.readline().decode('utf-8').strip()
                    logger.debug(f"Raw serial data: {line}")
                    
                    # Look for RED and IR values
                    if "RED=" in line and "IR=" in line:
                        try:
                            # Extract RED and IR values using regex
                            red_match = re.search(r'RED=(\d+)', line)
                            ir_match = re.search(r'IR=(\d+)', line)
                            
                            if red_match and ir_match:
                                red_value = int(red_match.group(1))
                                ir_value = int(ir_match.group(1))
                                
                                latest_reading.update({
                                    "red": red_value,
                                    "ir": ir_value,
                                    "timestamp": time.time(),
                                    "status": "Connected"
                                })
                                logger.info(f"Read values - RED: {red_value}, IR: {ir_value}")
                        except ValueError as e:
                            logger.warning(f"Failed to parse values: {e}")
                    else:
                        # Log other messages as debug info
                        logger.debug(f"Debug message: {line}")
                time.sleep(0.1)
    except Exception as e:
        logger.error(f"Serial error: {e}")
        latest_reading["status"] = "Disconnected"

def main():
    # Create the HTML file
    create_html_file()
    
    # Find the XIAO port
    port = find_xiao_port()
    if not port:
        logger.error("XIAO nRF52840 not found!")
        return

    # Start the HTTP server in a separate thread
    http_thread = threading.Thread(target=run_http_server)
    http_thread.daemon = True
    http_thread.start()

    # Start the WebSocket server in a separate thread
    ws_thread = threading.Thread(target=run_websocket_server)
    ws_thread.daemon = True
    ws_thread.start()

    # Give servers time to start
    time.sleep(1)

    # Start reading serial data
    logger.info("Starting serial data collection...")
    read_serial_data(port)

if __name__ == "__main__":
    main()