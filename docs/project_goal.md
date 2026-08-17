Create a small Rust + AI proof-of-concept app. A REST API that is called with some text input, and the output will be JSON containing information such as the evaluation results from an AI model. 
This is a future possibility for deploying similar applications on edge devices or elsewhere.


- Create a simple Rust application that performs text sentiment analysis using a pre-trained ONNX model. (I’ll leave this up to you; it can be a use case other than sentiment analysis, but choose a pre-trained model in ONNX format from available free repositories.) The libraries listed below are only recommendations; feel free to use any others. 

IN MORE DETAIL:

- Use a pre-trained ONNX model for sentiment analysis (e.g., DistilBERT SST-2).

- Tokenization: Use the tokenizers library to convert text into input tensors.

- Inference: Load the ONNX model using the ort library and perform inference on the tokenized input.

- REST API: Implement a simple POST /predict endpoint that accepts text and returns sentiment and a score.

- Output: Return JSON containing sentiment (positive/negative) and probability.

- Tests: Add 2 basic integration tests to verify the API’s functionality.