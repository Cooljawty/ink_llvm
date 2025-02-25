#include <stdio.h>
#include "wrapper.h"

int main() 
{
	printf("Initilizing:\n");
	void* story = Step(NULL); 

	printf("Stepping:\n");
	story = Step(story);

	return 0;
}
