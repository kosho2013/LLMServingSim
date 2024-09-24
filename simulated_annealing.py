import numpy as np
import random
import math
import matplotlib.pyplot as plt

# Generate random distance matrix for n cities
def generate_distance_matrix(num_cities):
    matrix = np.random.randint(10, 100, size=(num_cities, num_cities))
    np.fill_diagonal(matrix, 0)  # Distance from a city to itself is zero
    return matrix

# Calculate the total distance of a given tour
def calculate_total_distance(tour, distance_matrix):
    distance = 0
    for i in range(len(tour) - 1):
        distance += distance_matrix[tour[i], tour[i + 1]]
    # Add the distance to return to the starting city
    distance += distance_matrix[tour[-1], tour[0]]
    return distance

# Simulated Annealing algorithm
def simulated_annealing(distance_matrix, initial_temp, cooling_rate, num_iterations):
    num_cities = len(distance_matrix)
    
    # Step 1: Generate an initial random tour
    current_solution = list(range(num_cities))
    random.shuffle(current_solution)
    current_distance = calculate_total_distance(current_solution, distance_matrix)
    
    # Set the best solution as the current one initially
    best_solution = current_solution
    best_distance = current_distance

    # Track the temperature and best solution at each step
    temp = initial_temp
    distance_history = [current_distance]
    
    for iteration in range(num_iterations):
        # Step 2: Generate a neighbor solution by swapping two cities
        new_solution = current_solution[:]

        num_of_swapped = 2
        list_of_cities = random.sample(range(num_cities), num_of_swapped)
        for i in range(int(num_of_swapped/2)):
            new_solution[list_of_cities[i]], new_solution[list_of_cities[num_of_swapped-1-i]] = new_solution[list_of_cities[num_of_swapped-1-i]], new_solution[list_of_cities[i]]
        
        # Step 3: Calculate the new distance
        new_distance = calculate_total_distance(new_solution, distance_matrix)
        
        # Step 4: Accept the new solution with a probability
        if new_distance < current_distance:
            current_solution = new_solution
            current_distance = new_distance
        else:
            probability = math.exp((current_distance - new_distance) / temp)
            if random.random() < probability:
                current_solution = new_solution
                current_distance = new_distance
        
        # Step 5: Cool down the temperature
        temp *= cooling_rate
        
        # Track the best solution
        if current_distance < best_distance:
            best_solution = current_solution
            best_distance = current_distance
        
        # Track the history of the distance
        distance_history.append(current_distance)

    return best_solution, best_distance, distance_history

# Main
num_cities = 2080
distance_matrix = generate_distance_matrix(num_cities)
initial_temp = 1000
cooling_rate = 0.999
num_iterations = 100000

# Run Simulated Annealing
best_solution, best_distance, distance_history = simulated_annealing(distance_matrix, initial_temp, cooling_rate, num_iterations)

# Display results
print("Best Solution (Tour):", best_solution)
print("Best Distance:", best_distance)

# Plot the improvement over time
plt.plot(distance_history)
plt.title("Distance Over Time")
plt.xlabel("Iteration")
plt.ylabel("Total Distance")
plt.savefig('simulated_annealing.jpg')
