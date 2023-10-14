import matplotlib.pyplot as plt
import numpy as np
import json

# Read data from json file
file = open("webpages/fitness_graph.json", "r")
data = json.loads(file.read())

# Extract sigmoid data
herbi_sigmoid = data["herbi"]["Sigmoid"]
sigmoid_x = []
sigmoid_y = []
for i in range(len(herbi_sigmoid)):
    #log_x = np.log(herbi_sigmoid[i][0])
    #log_val = np.log(herbi_sigmoid[i][1])
    sigmoid_x.append(herbi_sigmoid[i][0])
    sigmoid_y.append(herbi_sigmoid[i][1])
    #sigmoid_x.append(log_x)
    #sigmoid_y.append(log_val)
    
# Extract relu data
herbi_relu = data["herbi"]["Relu"]
relu_x = []
relu_y = []
for i in range(len(herbi_relu)):
    #log_x = np.log(herbi_relu[i][0])
    #log_val = np.log(herbi_relu[i][1])
    relu_x.append(herbi_relu[i][0])
    relu_y.append(herbi_relu[i][1])
    #relu_x.append(log_x)
    #relu_y.append(log_val)


#plot fitness
plt.plot(sigmoid_x, sigmoid_y, label="Sigmoid")
plt.plot(relu_x, relu_y, label="Relu")
plt.title("Activation function vs fitness")
plt.xlabel("Total childen")
plt.ylabel("Average fitness")
plt.legend()
plt.show()