# Cave generation
### CA rules
* Lichen-esque: 
  * Survival: >= 4 neighbors
  * Birth: 3 or >= 7 (for filling holes, probably not necessary)
  * The idea here is seed with a semi-random box in the middle of the map, let 
  it grow until it touches and edge, and then find the bounds and use that as 
  the cave outline