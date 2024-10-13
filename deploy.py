from flask import Flask, jsonify, request
from flask_cors import CORS
import subprocess

app = Flask(__name__)
CORS(app)

@app.route('/deploy', methods=['GET', 'POST'])
def deploy_contract():
    # Define the Stellar CLI command
    command = [
        "stellar", "contract", "deploy",
        "--wasm", "target/wasm32-unknown-unknown/release/hello_world.wasm",
        "--source", "mediator",
        "--network", "testnet"
    ]
    
    try:
        # Run the command and capture stdout and stderr
        result = subprocess.run(command, stdout=subprocess.PIPE, stderr=subprocess.PIPE, text=True)
        
        # Check if the command was successful
        if result.returncode == 0:
            return jsonify({"success": True, "output": result.stdout})
        else:
            return jsonify({"success": False, "error": result.stderr}), 400
    except Exception as e:
        return jsonify({"success": False, "error": str(e)}), 500

if __name__ == '__main__':
    app.run(host='0.0.0.0', port=5000)
