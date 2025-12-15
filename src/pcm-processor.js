// class PCMProcessor extends AudioWorkletProcessor {
//   process(inputs) {
//     const input = inputs[0][0];
//     if (!input) return true;

//     const pcm = new Int16Array(input.length);
//     for (let i = 0; i < input.length; i++) {
//       pcm[i] = Math.max(-1, Math.min(1, input[i])) * 32767;
//     }

//     this.port.postMessage(pcm.buffer);
//     return true;
//   }
// }

// registerProcessor("pcm-processor", PCMProcessor);



class PCMProcessor extends AudioWorkletProcessor {
  process(inputs) {
    const input = inputs[0][0];
    if (!input) return true;

    const pcm = new Int16Array(input.length);
    for (let i = 0; i < input.length; i++) {
      pcm[i] = Math.max(-1, Math.min(1, input[i])) * 32767;
    }

    // ✅ ONLY postMessage — NO invoke here
    this.port.postMessage(pcm.buffer);

    return true;
  }
}

registerProcessor("pcm-processor", PCMProcessor);
