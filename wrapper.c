#include <stdio.h>
#include <stdbool.h>
#include "wrapper.h"

int main() 
{
	printf("Initilizing:\n");
	void* story = Step(NULL); 

	printf("Continue?: %s\n", CanContinue(story) ? "True":"False");

	printf("Stepping:\n");
	story = Step(story);
	
	printf("Status:\n\t%s", story == NULL ? "Ended" : "Unfinished");
	printf("\tContinue?: %s\n", CanContinue(story) ? "True":"False");

	return 0;
}
